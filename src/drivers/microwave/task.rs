/**
 * Microwave Control module
 *
 * Task owns all serial I/O. Extend here to:
 * - Implement protocol commands and acknowledgements.
 * - Add periodic polling or query support if available.
 * - Add reconnection handling when serial port disconnects.
 *
 * UI must never block; commands arrive via an mpsc channel and
 * state updates write into `Arc<RwLock<MicrowaveState>>` for fast snapshots.
 */

// handles commands to the microwave module

use tokio_serial::SerialStream;
use tokio::time::{sleep, Duration};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::config::config::{MicrowaveCommand, MicrowaveState};   
use crate::utilities::utils::open_microwave_connection;


pub async fn microwave_control(
    serial_port: &str,
    baud_rate: u32,
    mut microwave_rx: mpsc::Receiver<MicrowaveCommand>,
    state: Arc<RwLock<MicrowaveState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn: Option<SerialStream> = None;
    let mut tick = sleep(Duration::from_millis(200)); // ~5 Hz
    tokio::pin!(tick);

    loop {
        tokio::select! {
            Some(command) = microwave_rx.recv() => {
                match command {
                    MicrowaveCommand::Connect => {
                        match open_microwave_connection(serial_port, baud_rate).await {
                            Ok(stream) => {
                                conn = Some(stream);
                                let mut s = state.write().unwrap();
                                s.connected = true;
                                s.enabled = false;
                                s.last_error = None;
                                s.status = Some("connected".into());
                            }
                            Err(e) => {
                                let mut s = state.write().unwrap();
                                s.connected = false;
                                s.enabled = false;
                                s.last_error = Some(format!("connect failed: {}", e));
                                s.status = Some("disconnected".into());
                            }
                        }
                    }
                    MicrowaveCommand::Disconnect => {
                        conn = None;
                        let mut s = state.write().unwrap();
                        s.connected = false;
                        s.enabled = false;
                        s.status = Some("disconnected".into());
                    }
                    MicrowaveCommand::SetPower(watts) => {
                        let watts = watts.max(0.0);
                        let mut s = state.write().unwrap();
                        if conn.is_some() {
                            // For now, optimistic update; actual serial write omitted
                            s.power_watts = watts;
                            s.enabled = watts > 0.0;
                            s.last_error = None;
                            s.status = Some(if s.enabled { "heating".into() } else { "idle".into() });
                        } else {
                            s.last_error = Some("not connected".into());
                        }
                    }
                }
            }

            _ = &mut tick => {
                let mut s = state.write().unwrap();
                if s.connected {
                    if s.status.is_none() {
                        s.status = Some("Simulated/Unknown".into());
                    }
                }
                tick.as_mut().reset(tokio::time::Instant::now() + Duration::from_millis(200));
            }
        }
    }
}
