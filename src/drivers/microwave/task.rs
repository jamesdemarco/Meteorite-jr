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


pub async fn microwave_control(
    mut microwave: SerialStream,
    mut microwave_rx: mpsc::Receiver<MicrowaveCommand>,
    state: Arc<RwLock<MicrowaveState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tick = sleep(Duration::from_millis(200)); // ~5 Hz
    tokio::pin!(tick);

    loop {
        tokio::select! {
            Some(command) = microwave_rx.recv() => {
                match command {
                    MicrowaveCommand::Connect => {
                        let mut s = state.write().unwrap();
                        s.connected = true;
                        s.enabled = false;
                        s.last_error = None;
                        s.status = Some("connected".into());
                    }
                    MicrowaveCommand::Disconnect => {
                        let mut s = state.write().unwrap();
                        s.connected = false;
                        s.enabled = false;
                        s.status = Some("disconnected".into());
                    }
                    MicrowaveCommand::SetPower(watts) => {
                        let watts = watts.max(0.0);
                        // For now, optimistic update; actual serial write omitted
                        let mut s = state.write().unwrap();
                        s.power_watts = watts;
                        s.enabled = watts > 0.0;
                        s.last_error = None;
                        s.status = Some(if s.enabled { "heating".into() } else { "idle".into() });
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
