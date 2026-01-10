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
            .try_send(DuetCommand::Connect);
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
            .try_send(DuetCommand::Disconnect);
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

    fn send_gcode(&self, gcode: &str) {
        let msg = DuetCommand::SendGcode(gcode.to_owned());
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

    fn send_m_cmd(&self, m_cmd: &str) {
        let msg = DuetCommand::SendMCommand(m_cmd.to_owned());
        let send_res = self.cmd_tx.try_send(msg);
        let mut s = self.state.write().unwrap();
        match send_res {
            Ok(_) => {
                s.last_error = None;
                s.last_command = Some(m_cmd.to_owned());
            }
            Err(e) => {
                s.last_error = Some(format!("send failed: {}", e));
            }
        }
    }
}
