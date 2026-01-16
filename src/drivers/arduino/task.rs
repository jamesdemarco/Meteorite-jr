/**
 * Arduino Pneumatic Pressure Control Driver
 *
 * Task owns all serial I/O to Arduino for pneumatic pressure control.
 * - Starts disconnected
 * - On Connect: opens serial port at specified baud rate
 * - Polls pressure telemetry at 10 Hz when connected
 * - Applies pressure setpoints and enable/disable commands
 * - Updates ArduinoState in Arc<RwLock<ArduinoState>> for UI snapshots
 *
 * Protocol (newline-delimited ASCII):
 * - Outgoing commands:
 *   "SET_PSI <float>\n"  - Set pressure setpoint
 *   "ENABLE <0|1>\n"     - Enable/disable pressure control
 *   "READ\n"             - Request telemetry (optional, can also just listen)
 * - Incoming telemetry (expected formats):
 *   "P PSI=<float> MA=<float> OK=<0|1>\n"  - Full telemetry
 *   "PSI=<float>\n"                         - Minimal telemetry
 *
 * TODO: Adjust protocol parsing if Arduino firmware differs from above.
 *
 * UI must never block; commands arrive via mpsc channel and
 * state updates write into `Arc<RwLock<ArduinoState>>` for fast snapshots.
 */

use tokio::time::{sleep, Duration, Instant};
use std::sync::{Arc, RwLock};
use std::io::{BufRead, BufReader, Write};
use tokio::sync::mpsc;

use crate::config::config::{ArduinoCommand, ArduinoState};

/// Parse Arduino telemetry line
/// Accepts formats:
/// - "P PSI=34.7 MA=12.3 OK=1"
/// - "PSI=34.7"
/// Returns (psi, ma, ok)
fn parse_telemetry(line: &str) -> Option<(f32, Option<f32>, Option<bool>)> {
    let line = line.trim();
    
    // Try to find PSI value
    let psi = if let Some(pos) = line.find("PSI=") {
        let rest = &line[pos + 4..];
        let end = rest.find(|c: char| !c.is_numeric() && c != '.' && c != '-')
            .unwrap_or(rest.len());
        rest[..end].parse::<f32>().ok()?
    } else {
        return None;
    };
    
    // Try to find MA value
    let ma = if let Some(pos) = line.find("MA=") {
        let rest = &line[pos + 3..];
        let end = rest.find(|c: char| !c.is_numeric() && c != '.' && c != '-')
            .unwrap_or(rest.len());
        rest[..end].parse::<f32>().ok()
    } else {
        None
    };
    
    // Try to find OK value
    let ok = if let Some(pos) = line.find("OK=") {
        let rest = &line[pos + 3..];
        let end = rest.find(|c: char| !c.is_numeric())
            .unwrap_or(rest.len());
        rest[..end].parse::<u8>().ok().map(|v| v != 0)
    } else {
        None
    };
    
    Some((psi, ma, ok))
}

pub async fn arduino_control(
    port: &str,
    baud: u32,
    mut arduino_rx: mpsc::Receiver<ArduinoCommand>,
    state: Arc<RwLock<ArduinoState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    use serialport::SerialPort;
    
    let mut serial_port: Option<Box<dyn SerialPort>> = None;
    let mut connected = false;
    let mut last_poll = Instant::now();
    let poll_interval = Duration::from_millis(100); // 10 Hz
    
    loop {
        // Check if it's time to poll
        let should_poll = connected && last_poll.elapsed() >= poll_interval;
        
        // Select between command and poll timing
        tokio::select! {
            Some(command) = arduino_rx.recv() => {
                match command {
                    ArduinoCommand::Connect => {
                        // Open serial port in blocking task
                        let port_name = port.to_string();
                        let baud_rate = baud;
                        
                        match tokio::task::spawn_blocking(move || {
                            serialport::new(&port_name, baud_rate)
                                .timeout(Duration::from_millis(100))
                                .open()
                        }).await {
                            Ok(Ok(port)) => {
                                serial_port = Some(port);
                                connected = true;
                                last_poll = Instant::now();
                                let mut s = state.write().unwrap();
                                s.connected = true;
                                s.enabled = false;
                                s.last_error = None;
                                s.status = Some("connected".into());
                                s.pressure_measured_psi = 0.0;
                                s.loop_current_ma = None;
                                s.signal_ok = None;
                            }
                            Ok(Err(e)) => {
                                let mut s = state.write().unwrap();
                                s.connected = false;
                                s.last_error = Some(format!("serial open failed: {}", e));
                                s.status = Some("error".into());
                            }
                            Err(e) => {
                                let mut s = state.write().unwrap();
                                s.connected = false;
                                s.last_error = Some(format!("spawn_blocking failed: {}", e));
                                s.status = Some("error".into());
                            }
                        }
                    }
                    ArduinoCommand::Disconnect => {
                        serial_port = None;
                        connected = false;
                        let mut s = state.write().unwrap();
                        s.connected = false;
                        s.enabled = false;
                        s.status = Some("disconnected".into());
                        s.pressure_measured_psi = 0.0;
                        s.loop_current_ma = None;
                        s.signal_ok = None;
                    }
                    ArduinoCommand::Enable(enable) => {
                        if connected {
                            if let Some(ref mut port) = serial_port {
                                let cmd = format!("ENABLE {}\n", if enable { 1 } else { 0 });
                                let port_clone = port.try_clone();
                                
                                match port_clone {
                                    Ok(mut p) => {
                                        match tokio::task::spawn_blocking(move || {
                                            p.write_all(cmd.as_bytes())
                                        }).await {
                                            Ok(Ok(_)) => {
                                                let mut s = state.write().unwrap();
                                                s.enabled = enable;
                                                s.last_error = None;
                                                s.status = Some(if enable { "enabled" } else { "disabled" }.into());
                                            }
                                            Ok(Err(e)) => {
                                                let mut s = state.write().unwrap();
                                                s.last_error = Some(format!("write failed: {}", e));
                                                s.status = Some("error".into());
                                                connected = false;
                                                s.connected = false;
                                            }
                                            Err(e) => {
                                                let mut s = state.write().unwrap();
                                                s.last_error = Some(format!("spawn failed: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let mut s = state.write().unwrap();
                                        s.last_error = Some(format!("port clone failed: {}", e));
                                    }
                                }
                            }
                        } else {
                            let mut s = state.write().unwrap();
                            s.last_error = Some("not connected".into());
                        }
                    }
                    ArduinoCommand::SetPressureSetpoint(psi) => {
                        // Update setpoint in state
                        {
                            let mut s = state.write().unwrap();
                            s.pressure_setpoint_psi = psi.max(0.0);
                        }
                        
                        // If enabled and connected, send command to Arduino
                        let is_enabled = state.read().unwrap().enabled;
                        if connected && is_enabled {
                            if let Some(ref mut port) = serial_port {
                                let cmd = format!("SET_PSI {:.2}\n", psi);
                                let port_clone = port.try_clone();
                                
                                match port_clone {
                                    Ok(mut p) => {
                                        match tokio::task::spawn_blocking(move || {
                                            p.write_all(cmd.as_bytes())
                                        }).await {
                                            Ok(Ok(_)) => {
                                                let mut s = state.write().unwrap();
                                                s.last_error = None;
                                            }
                                            Ok(Err(e)) => {
                                                let mut s = state.write().unwrap();
                                                s.last_error = Some(format!("write failed: {}", e));
                                                s.status = Some("error".into());
                                                connected = false;
                                                s.connected = false;
                                            }
                                            Err(e) => {
                                                let mut s = state.write().unwrap();
                                                s.last_error = Some(format!("spawn failed: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let mut s = state.write().unwrap();
                                        s.last_error = Some(format!("port clone failed: {}", e));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ = sleep(poll_interval), if should_poll => {
                // Time to poll for telemetry
                if let Some(ref mut port) = serial_port {
                    last_poll = Instant::now();
                    
                    // Send READ command
                    let port_clone = port.try_clone();
                    if let Ok(mut p) = port_clone {
                        // Send READ and read response in blocking task
                        match tokio::task::spawn_blocking(move || -> Result<String, std::io::Error> {
                            p.write_all(b"READ\n")?;
                            p.flush()?;
                            
                            let mut reader = BufReader::new(p);
                            let mut line = String::new();
                            reader.read_line(&mut line)?;
                            Ok(line)
                        }).await {
                            Ok(Ok(line)) => {
                                // Parse telemetry
                                if let Some((psi, ma, ok)) = parse_telemetry(&line) {
                                    let mut s = state.write().unwrap();
                                    s.pressure_measured_psi = psi;
                                    s.loop_current_ma = ma;
                                    s.signal_ok = ok;
                                    s.last_error = None;
                                }
                            }
                            Ok(Err(e)) => {
                                let mut s = state.write().unwrap();
                                s.last_error = Some(format!("read failed: {}", e));
                                s.status = Some("error".into());
                                connected = false;
                                s.connected = false;
                            }
                            Err(e) => {
                                let mut s = state.write().unwrap();
                                s.last_error = Some(format!("spawn failed: {}", e));
                            }
                        }
                    }
                }
            }
        }
    }
}
