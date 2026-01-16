#[cfg(feature = "mock")]
mod mock;
#[cfg(feature = "mock")]
pub use mock::MockArduino;

#[cfg(not(feature = "mock"))]
mod client;
#[cfg(not(feature = "mock"))]
pub use client::ArduinoClient;
