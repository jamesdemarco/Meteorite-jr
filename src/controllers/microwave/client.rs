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
            .try_send(MicrowaveCommand::Connect);
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
                s.status = Some("connecting...".into());
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn disconnect(&self) {
        let send_res = self
            .cmd_tx
            .try_send(MicrowaveCommand::Disconnect);
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.status = Some("disconnecting...".into());
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn set_power(&self, watts: f32) {
        let msg = MicrowaveCommand::SetPowerWatts(watts);
        let send_res = self.cmd_tx.try_send(msg);
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
                s.power_watts = watts.max(0.0);
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn set_frequency(&self, hz: i32) {
        let msg = MicrowaveCommand::SetFrequencyHz(hz);
        let send_res = self.cmd_tx.try_send(msg);
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn rf_on(&self) {
        let send_res = self.cmd_tx.try_send(MicrowaveCommand::RfOn);
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
                s.enabled = true;
                s.status = Some("RF on".into());
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn rf_off(&self) {
        let send_res = self.cmd_tx.try_send(MicrowaveCommand::RfOff);
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
                s.enabled = false;
                s.status = Some("RF off".into());
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
