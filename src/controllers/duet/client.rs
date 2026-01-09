use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::config::config::{DuetCommand, DuetState};
use crate::controllers::DuetController;

pub struct DuetClient {
    cmd_tx: mpsc::Sender<DuetCommand>,
    state: Arc<RwLock<DuetState>>, // cached state
}

impl DuetClient {
    pub fn new(cmd_tx: mpsc::Sender<DuetCommand>, state: Arc<RwLock<DuetState>>) -> Self {
        Self { cmd_tx, state }
    }

    pub fn state_handle(&self) -> Arc<RwLock<DuetState>> {
        Arc::clone(&self.state)
    }
}

impl DuetController for DuetClient {
    fn connect(&self) {
        let send_res = self
            .cmd_tx
            .try_send(DuetCommand { command: "CONNECT".into() });
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.connected = true;
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
            .try_send(DuetCommand { command: "DISCONNECT".into() });
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.connected = false;
                s.status = Some("disconnected".into());
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn send_gcode(&self, gcode: &str) {
        let msg = DuetCommand {
            command: gcode.to_owned(),
        };
        let send_res = self.cmd_tx.try_send(msg);
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
                s.last_command = Some(gcode.to_owned());
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }

    fn state(&self) -> DuetState {
        self.state.read().unwrap().clone()
    }
}
