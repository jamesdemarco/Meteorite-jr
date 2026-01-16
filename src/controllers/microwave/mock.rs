use std::sync::Mutex;

use crate::config::config::MicrowaveState;
use crate::controllers::MicrowaveController;

pub struct MockMicrowave {
    state: Mutex<MicrowaveState>,
}

impl MockMicrowave {
    pub fn new() -> Self {
        let mut s = MicrowaveState::default();
        s.connected = false;
        s.enabled = false;
        s.power_watts = 0.0;
        Self { state: Mutex::new(s) }
    }
}

impl MicrowaveController for MockMicrowave {
    fn connect(&self) {
        let mut s = self.state.lock().unwrap();
        s.connected = true;
        s.last_error = None;
        s.status = Some("connected".into());
    }

    fn disconnect(&self) {
        let mut s = self.state.lock().unwrap();
        s.connected = false;
        s.enabled = false;
        s.status = Some("disconnected".into());
    }

    fn set_power(&self, watts: f32) {
        let mut s = self.state.lock().unwrap();
        if !s.connected {
            s.last_error = Some("Microwave not connected".into());
            return;
        }
        s.last_error = None;
        s.power_watts = watts.max(0.0);
    }

    fn set_frequency(&self, _hz: i32) {
        let mut s = self.state.lock().unwrap();
        if !s.connected {
            s.last_error = Some("Microwave not connected".into());
            return;
        }
        s.last_error = None;
        // Mock doesn't actually track frequency
    }

    fn rf_on(&self) {
        let mut s = self.state.lock().unwrap();
        if !s.connected {
            s.last_error = Some("Microwave not connected".into());
            return;
        }
        s.last_error = None;
        s.enabled = true;
        s.status = Some("RF on".into());
    }

    fn rf_off(&self) {
        let mut s = self.state.lock().unwrap();
        if !s.connected {
            s.last_error = Some("Microwave not connected".into());
            return;
        }
        s.last_error = None;
        s.enabled = false;
        s.status = Some("RF off".into());
    }

    fn state(&self) -> MicrowaveState {
        self.state.lock().unwrap().clone()
    }
}
