/****************    Microwave Control module   *************/

// handles commands to the microwave module

use tokio_serial::SerialStream;
use tokio::time::{sleep, Duration};
use std::sync::{Arc, RwLock};
use tokio::sync::watch;

use crate::config::config::{MicrowaveCommand, MicrowaveState};   


pub async fn microwave_control(
    mut microwave: SerialStream,
    mut microwave_rx: watch::Receiver<MicrowaveCommand>,
    state: Arc<RwLock<MicrowaveState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tick = sleep(Duration::from_millis(200)); // ~5 Hz
    tokio::pin!(tick);

    loop {
        tokio::select! {
            changed = microwave_rx.changed() => {
                if changed.is_ok() {
                    let command = microwave_rx.borrow().clone();
                    let cmd = command.command.trim().to_string();
                    if cmd.eq_ignore_ascii_case("CONNECT") {
                        let mut s = state.write().unwrap();
                        s.connected = true;
                        s.enabled = false;
                        s.last_error = None;
                        s.status = Some("connected".into());
                    } else if cmd.eq_ignore_ascii_case("DISCONNECT") {
                        let mut s = state.write().unwrap();
                        s.connected = false;
                        s.enabled = false;
                        s.status = Some("disconnected".into());
                    } else if let Some(val) = cmd.strip_prefix("SET_POWER ") {
                        let watts = val.parse::<f32>().unwrap_or(0.0).max(0.0);
                        // For now, optimistic update; actual serial write omitted
                        let mut s = state.write().unwrap();
                        s.power_watts = watts;
                        s.enabled = watts > 0.0;
                        s.last_error = None;
                        s.status = Some(if s.enabled { "heating".into() } else { "idle".into() });
                    } else {
                        // Unknown command
                        let mut s = state.write().unwrap();
                        s.last_error = Some("Unknown command".into());
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
