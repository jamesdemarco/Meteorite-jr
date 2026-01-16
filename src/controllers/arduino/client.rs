use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::config::config::{ArduinoCommand, ArduinoState};
use crate::controllers::ArduinoController;

pub struct ArduinoClient {
    cmd_tx: mpsc::Sender<ArduinoCommand>,
    state: Arc<RwLock<ArduinoState>>, // cached state
}

impl ArduinoClient {
    pub fn new(cmd_tx: mpsc::Sender<ArduinoCommand>, state: Arc<RwLock<ArduinoState>>) -> Self {
        Self { cmd_tx, state }
    }

    pub fn state_handle(&self) -> Arc<RwLock<ArduinoState>> {
        Arc::clone(&self.state)
    }
}

impl ArduinoController for ArduinoClient {
    fn connect(&self) {
        let send_res = self.cmd_tx.try_send(ArduinoCommand::Connect);
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
        let send_res = self.cmd_tx.try_send(ArduinoCommand::Disconnect);
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

    fn enable(&self, enable: bool) {
        let send_res = self.cmd_tx.try_send(ArduinoCommand::Enable(enable));
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
                s.enabled = enable;
                s.status = Some(if enable { "enabled" } else { "disabled" }.into());
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn set_pressure_setpoint(&self, psi: f32) {
        let send_res = self.cmd_tx.try_send(ArduinoCommand::SetPressureSetpoint(psi));
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
                s.pressure_setpoint_psi = psi.max(0.0);
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn state(&self) -> ArduinoState {
        self.state.read().unwrap().clone()
    }
}
