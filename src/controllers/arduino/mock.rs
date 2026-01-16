use std::sync::Mutex;

use crate::config::config::ArduinoState;
use crate::controllers::ArduinoController;

pub struct MockArduino {
    state: Mutex<ArduinoState>,
}

impl MockArduino {
    pub fn new() -> Self {
        let mut s = ArduinoState::default();
        s.connected = false;
        s.enabled = false;
        s.pressure_setpoint_psi = 0.0;
        s.pressure_measured_psi = 0.0;
        Self { state: Mutex::new(s) }
    }
}

impl ArduinoController for MockArduino {
    fn connect(&self) {
        let mut s = self.state.lock().unwrap();
        s.connected = true;
        s.last_error = None;
        s.status = Some("connected (mock)".into());
        // Simulate some reasonable pressure reading
        s.pressure_measured_psi = 0.0;
    }

    fn disconnect(&self) {
        let mut s = self.state.lock().unwrap();
        s.connected = false;
        s.enabled = false;
        s.status = Some("disconnected".into());
        s.pressure_measured_psi = 0.0;
    }

    fn enable(&self, enable: bool) {
        let mut s = self.state.lock().unwrap();
        if !s.connected {
            s.last_error = Some("Arduino not connected".into());
            return;
        }
        s.last_error = None;
        s.enabled = enable;
        s.status = Some(if enable { "enabled" } else { "disabled" }.into());
    }

    fn set_pressure_setpoint(&self, psi: f32) {
        let mut s = self.state.lock().unwrap();
        if !s.connected {
            s.last_error = Some("Arduino not connected".into());
            return;
        }
        s.last_error = None;
        s.pressure_setpoint_psi = psi.max(0.0);
        
        // Mock behavior: simulate pressure tracking setpoint with some lag
        if s.enabled {
            // Simulate pressure approaching setpoint
            s.pressure_measured_psi = psi * 0.9; // 90% of setpoint in mock
            s.loop_current_ma = Some(12.0 + psi * 0.5); // Simulate some current based on pressure
            s.signal_ok = Some(true);
        }
    }

    fn state(&self) -> ArduinoState {
        self.state.lock().unwrap().clone()
    }
}
