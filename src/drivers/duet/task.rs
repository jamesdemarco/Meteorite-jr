/**
 * DUET Control module
 *
 * Task owns all hardware I/O. Uses HTTP RRF3 endpoints instead of TCP/Telnet.
 * - Uses reqwest::Client for HTTP requests
 * - Polls rr_status for position and connection status
 * - Sends G-code via rr_gcode endpoint
 *
 * UI must never block; commands arrive via an mpsc channel and
 * state updates write into `Arc<RwLock<DuetState>>` for fast snapshots.
 */

// handles communication with the DUET 2 board
// manages G-code sending and status receiving via HTTP RRF3

use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use serde::Deserialize;

use crate::config::config::{DuetCommand, DuetState};

#[derive(Deserialize, Debug)]
struct RrStatus {
    status: String,
    coords: Coords,
}

#[derive(Deserialize, Debug)]
struct Coords {
    xyz: [f32; 3],
    #[serde(default)]
    machine: [f32; 3],
}

pub async fn duet_control(
    duet_ip: &str,
    mut rx: mpsc::Receiver<DuetCommand>,
    state: Arc<RwLock<DuetState>>,
) {
    let client = reqwest::Client::new();
    let mut connected = false;
    let mut poll_interval = interval(Duration::from_millis(150)); // ~6-7 Hz

    loop {
        tokio::select! {
            // Handle incoming commands
            Some(cmd) = rx.recv() => {
                match cmd {
                    DuetCommand::Connect => {
                        match client.get(crate::utilities::utils::rr_status_url(duet_ip)).send().await {
                            Ok(_) => {
                                connected = true;
                                let mut s = state.write().unwrap();
                                s.connected = true;
                                s.status = Some("connected".to_string());
                                s.last_error = None;
                            }
                            Err(e) => {
                                connected = false;
                                let mut s = state.write().unwrap();
                                s.connected = false;
                                s.status = Some("connection failed".to_string());
                                s.last_error = Some(format!("Connect error: {}", e));
                            }
                        }
                    }
                    DuetCommand::Disconnect => {
                        connected = false;
                        let mut s = state.write().unwrap();
                        s.connected = false;
                        s.status = Some("disconnected".to_string());
                    }
                    DuetCommand::SendGcode(gcode) => {
                        if !connected {
                            let mut s = state.write().unwrap();
                            s.last_error = Some("not connected".to_string());
                        } else {
                            let url = crate::utilities::utils::rr_gcode_url(duet_ip, &gcode);
                            match client.get(&url).send().await {
                                Ok(_) => {
                                    let mut s = state.write().unwrap();
                                    s.last_command = Some(gcode.clone());
                                    s.last_error = None;
                                }
                                Err(e) => {
                                    connected = false;
                                    let mut s = state.write().unwrap();
                                    s.status = Some("error".to_string());
                                    s.last_error = Some(format!("Gcode error: {}", e));
                                    s.connected = false;
                                }
                            }
                        }
                    }
                    DuetCommand::SendMCommand(m_cmd) => {
                        if !connected {
                            let mut s = state.write().unwrap();
                            s.last_error = Some("not connected".to_string());
                        } else {
                            let url = crate::utilities::utils::rr_gcode_url(duet_ip, &m_cmd);
                            match client.get(&url).send().await {
                                Ok(_) => {
                                    let mut s = state.write().unwrap();
                                    s.last_command = Some(m_cmd.clone());
                                    s.last_error = None;
                                }
                                Err(e) => {
                                    connected = false;
                                    let mut s = state.write().unwrap();
                                    s.status = Some("error".to_string());
                                    s.last_error = Some(format!("M-command error: {}", e));
                                    s.connected = false;
                                }
                            }
                        }
                    }
                }
            }

            // Polling tick
            _ = poll_interval.tick() => {
                if connected {
                    match client.get(crate::utilities::utils::rr_status_url(duet_ip)).send().await {
                        Ok(resp) => {
                            match resp.json::<RrStatus>().await {
                                Ok(status) => {
                                    let mut s = state.write().unwrap();
                                    s.position = status.coords.xyz;
                                    s.status = Some(match status.status.as_str() {
                                        "I" => "idle".to_string(),
                                        "P" => "printing".to_string(),
                                        "S" => "stopped".to_string(),
                                        "H" => "halted".to_string(),
                                        "D" => "pausing".to_string(),
                                        other => other.to_string(),
                                    });
                                    s.last_error = None;
                                }
                                Err(e) => {
                                    connected = false;
                                    let mut s = state.write().unwrap();
                                    s.connected = false;
                                    s.status = Some("parse error".to_string());
                                    s.last_error = Some(format!("JSON parse: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            connected = false;
                            let mut s = state.write().unwrap();
                            s.connected = false;
                            s.status = Some("offline".to_string());
                            s.last_error = Some(format!("Poll error: {}", e));
                        }
                    }
                }
            }
        }
    }
}
