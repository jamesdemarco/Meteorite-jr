pub mod duet;
pub mod microwave;
pub mod arduino;

// Trait interfaces for non-blocking UI calls
// Command methods enqueue work; query methods return cached state.
use crate::config::config::{DuetState, MicrowaveState, ArduinoState};

/// Controllers must be non-blocking:
/// - Command methods enqueue work and return immediately.
/// - No hardware I/O or awaits inside these methods.
/// - State queries return a cheap, cloned snapshot from cached state.
pub trait DuetController: Send + Sync {
	// Toggle connection state (no I/O in mock)
	fn connect(&self);
	fn disconnect(&self);
	// Fire-and-forget: enqueue a G-code command; returns immediately.
	fn send_gcode(&self, gcode: &str);
	// Snapshot of cached duet state.
	fn state(&self) -> DuetState;
    fn send_m_cmd(&self, m_cmd: &str);
}

/// Same non-blocking rules apply to the Microwave controller.
pub trait MicrowaveController: Send + Sync {
	// Toggle connection state (no I/O in mock)
	fn connect(&self);
	fn disconnect(&self);
	// Fire-and-forget: set microwave power in watts; returns immediately.
	fn set_power(&self, watts: f32);
	// Fire-and-forget: set microwave frequency in Hz; returns immediately.
	fn set_frequency(&self, hz: i32);
	// RF control: turn RF on/off
	fn rf_on(&self);
	fn rf_off(&self);
	// Snapshot of cached microwave state.
	fn state(&self) -> MicrowaveState;
}

/// Arduino pneumatic pressure controller (non-blocking)
pub trait ArduinoController: Send + Sync {
	// Toggle connection state (no I/O in mock)
	fn connect(&self);
	fn disconnect(&self);
	// Enable/disable pressure control
	fn enable(&self, enable: bool);
	// Fire-and-forget: set pressure setpoint in PSI; returns immediately.
	fn set_pressure_setpoint(&self, psi: f32);
	// Snapshot of cached Arduino state.
	fn state(&self) -> ArduinoState;
}
