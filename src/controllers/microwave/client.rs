use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::config::config::{MicrowaveCommand, MicrowaveState};
use crate::controllers::MicrowaveController;

pub struct MicrowaveClient {
    cmd_tx: mpsc::Sender<MicrowaveCommand>,
    state: Arc<RwLock<MicrowaveState>>, // cached state
}

impl MicrowaveClient {
    pub fn new(cmd_tx: mpsc::Sender<MicrowaveCommand>, state: Arc<RwLock<MicrowaveState>>) -> Self {
        Self { cmd_tx, state }
    }

    pub fn state_handle(&self) -> Arc<RwLock<MicrowaveState>> {
        Arc::clone(&self.state)
    }
}

impl MicrowaveController for MicrowaveClient {
    fn connect(&self) {
        let send_res = self
            .cmd_tx
            .try_send(MicrowaveCommand { command: "CONNECT".into() });
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.connected = true;
                s.enabled = false;
                s.last_error = None;
                s.status = Some("connected".into());
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn disconnect(&self) {
        let send_res = self
            .cmd_tx
            .try_send(MicrowaveCommand { command: "DISCONNECT".into() });
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.connected = false;
                s.enabled = false;
                s.status = Some("disconnected".into());
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn set_power(&self, watts: f32) {
        let msg = MicrowaveCommand {
            command: format!("SET_POWER {}", watts),
        };
        let send_res = self.cmd_tx.try_send(msg);
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
                s.power_watts = watts.max(0.0);
                s.enabled = s.power_watts > 0.0;
                s.status = Some(if s.enabled { "heating".into() } else { "idle".into() });
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn state(&self) -> MicrowaveState {
        self.state.read().unwrap().clone()
    }
}
