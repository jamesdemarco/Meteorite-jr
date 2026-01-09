use std::sync::Mutex;

use crate::config::config::DuetState;
use crate::controllers::DuetController;

pub struct MockDuet {
    state: Mutex<DuetState>,
}

impl MockDuet {
    pub fn new() -> Self {
        let mut s = DuetState::default();
        s.connected = false;
        Self { state: Mutex::new(s) }
    }
}

impl DuetController for MockDuet {
    fn connect(&self) {
        let mut s = self.state.lock().unwrap();
        s.connected = true;
        s.last_error = None;
        s.status = Some("connected".into());
    }

    fn disconnect(&self) {
        let mut s = self.state.lock().unwrap();
        s.connected = false;
        s.status = Some("disconnected".into());
    }

    fn send_gcode(&self, gcode: &str) {
        let mut s = self.state.lock().unwrap();
        if !s.connected {
            s.last_error = Some("Duet not connected".into());
            return;
        }
        s.last_error = None;
        s.last_command = Some(gcode.to_string());
        s.status = Some("busy".into());
        // very simple simulation: G28 homes, G0/G1 moves; parse X/Y/Z values
        let g = gcode.trim().to_uppercase();
        if g.starts_with("G28") {
            s.position = [0.0, 0.0, 0.0];
            s.status = Some("idle".into());
            return;
        }
        if g.starts_with("G0") || g.starts_with("G1") {
            // parse tokens like X12.3 Y-1 Z0.5
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
            return;
        }
        // unknown command: just mark idle
        s.status = Some("idle".into());
    }

    fn state(&self) -> DuetState {
        self.state.lock().unwrap().clone()
    }
}
