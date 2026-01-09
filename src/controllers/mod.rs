pub mod duet;
pub mod microwave;

// Trait interfaces for non-blocking UI calls
// Command methods enqueue work; query methods return cached state.
use crate::config::config::{DuetState, MicrowaveState};

pub trait DuetController: Send + Sync {
	// Toggle connection state (no I/O in mock)
	fn connect(&self);
	fn disconnect(&self);
	// Fire-and-forget: enqueue a G-code command; returns immediately.
	fn send_gcode(&self, gcode: &str);
	// Snapshot of cached duet state.
	fn state(&self) -> DuetState;
}

pub trait MicrowaveController: Send + Sync {
	// Toggle connection state (no I/O in mock)
	fn connect(&self);
	fn disconnect(&self);
	// Fire-and-forget: set microwave power in watts; returns immediately.
	fn set_power(&self, watts: f32);
	// Snapshot of cached microwave state.
	fn state(&self) -> MicrowaveState;
}
