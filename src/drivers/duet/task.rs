/**
 * DUET Control module
 *
 * Task owns all hardware I/O. Extend here to:
 * - Parse protocol responses and update `DuetState` fields.
 * - Add reconnection logic on errors/timeouts.
 * - Add polling at 5–10 Hz to refresh position/status.
 *
 * UI must never block; commands arrive via an mpsc channel and
 * state updates write into `Arc<RwLock<DuetState>>` for fast snapshots.
 */

// handles communication with the DUET 2 board
// manages G-code sending and status receiving

// use reqwest
use tokio::time::{sleep, Duration};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use crate::config::config::{DuetCommand, DuetState};

use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
pub async fn duet_control(
    mut duet_connection: TcpStream,
    mut duet_rx: mpsc::Receiver<DuetCommand>,
    state: Arc<RwLock<DuetState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tick = sleep(Duration::from_millis(150)); // ~6-7 Hz
    tokio::pin!(tick);

    loop {
        tokio::select! {
            // Handle incoming commands
            Some(command) = duet_rx.recv() => {
                match command {
                    DuetCommand::Connect => {
                        let mut s = state.write().unwrap();
                        s.connected = true;
                        s.last_error = None;
                        s.status = Some("connected".into());
                    }
                    DuetCommand::Disconnect => {
                        let mut s = state.write().unwrap();
                        s.connected = false;
                        s.status = Some("disconnected".into());
                    }
                    DuetCommand::SendGcode(gcode) => {
                        // Treat as a G-code send
                        let write_res = duet_connection.write_all(gcode.as_bytes()).await;
                        let mut s = state.write().unwrap();
                        match write_res {
                            Ok(_) => {
                                s.last_error = None;
                                s.last_command = Some(gcode.clone());
                                s.status = Some("busy".into());
                                // optimistic position update on simple G0/G1/G28
                                let g = gcode.to_uppercase();
                                if g.starts_with("G28") {
                                    s.position = [0.0, 0.0, 0.0];
                                    s.status = Some("idle".into());
                                } else if g.starts_with("G0") || g.starts_with("G1") {
                                    let mut pos = s.position;
                                    for tok in g.split_whitespace() {
                                        if let Some(val) = tok.strip_prefix('X') {
                                            if let Ok(v) = val.parse::<f32>() { pos[0] = v; }
                                        } else if let Some(val) = tok.strip_prefix('Y') {
                                            if let Ok(v) = val.parse::<f32>() { pos[1] = v; }
                                        } else if let Some(val) = tok.strip_prefix('Z') {
                                            if let Ok(v) = val.parse::<f32>() { pos[2] = v; }
                                        }
                                    }
                                    s.position = pos;
                                    s.status = Some("idle".into());
                                } else {
                                    s.status = Some("idle".into());
                                }
                            }
                            Err(e) => {
                                s.last_error = Some(format!("I/O error: {}", e));
                                s.connected = false;
                                s.status = Some("error".into());
                            }
                        }
                    }
                    DuetCommand::SendMCommand(m_cmd) => {
                        // Treat as an M-command send
                        let write_res = duet_connection.write_all(m_cmd.as_bytes()).await;
                        let mut s = state.write().unwrap();
                        match write_res {
                            Ok(_) => {
                                s.last_error = None;
                                s.last_command = Some(m_cmd.clone());
                                s.status = Some("busy".into());
                                s.status = Some("idle".into());
                            }
                            Err(e) => {
                                s.last_error = Some(format!("I/O error: {}", e));
                                s.connected = false;
                                s.status = Some("error".into());
                            }
                        }
                    }
                }
            }

            // Periodic status update (simulated)
            _ = &mut tick => {
                let mut s = state.write().unwrap();
                if s.connected {
                    // Without protocol detail, mark as simulated/unknown
                    if s.status.is_none() || s.status.as_deref() == Some("idle") {
                        s.status = Some("Simulated/Unknown".into());
                    }
                }
                tick.as_mut().reset(tokio::time::Instant::now() + Duration::from_millis(150));
            }
        }
    }
}
