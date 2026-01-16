

pub const duet_ip: &str = "192.168.10.2";

pub const MICROWAVE_SERIAL_PORT: &str = "/dev/ttyUSB0";
pub const MICROWAVE_BAUD_RATE: u32 = 9600;
pub const MICROCIRCUITS_VENDOR_ID: u16 = 0xFFFF;
pub const MICROCIRCUITS_PRODUCT_ID: u16 = 0xFFFF;

pub const ARDUINO_SERIAL_PORT: &str = "COM5"; // TODO: Configure for your hardware
pub const ARDUINO_BAUD_RATE: u32 = 115200;

// Helper function to create TargetProperties for MiniCircuit driver
pub fn build_target_properties() -> minicircuit_commands::properties::TargetProperties {
    use minicircuit_commands::properties::{VendorId, ProductId};
    use minicircuit_commands::prelude::BaudRate;
    minicircuit_commands::properties::TargetProperties {
        vendor_id: VendorId { vendor_id: MICROCIRCUITS_VENDOR_ID },
        product_id: ProductId { product_id: MICROCIRCUITS_PRODUCT_ID },
        port: Some(MICROWAVE_SERIAL_PORT.to_string()),
        baud_rate: BaudRate { baud_rate: MICROWAVE_BAUD_RATE },
        ..Default::default()
    }
}


#[derive(Clone, Debug)]
pub enum DuetCommand {
    Connect,
    Disconnect,
    SendGcode(String),
    SendMCommand(String),
}


#[derive(Clone, Debug)]
pub enum MicrowaveCommand {
    Connect,
    Disconnect,
    RfOn,
    RfOff,
    SetPowerWatts(f32),
    SetFrequencyHz(i32),
}

#[derive(Clone, Debug)]
pub enum ArduinoCommand {
    Connect,
    Disconnect,
    Enable(bool),
    SetPressureSetpoint(f32),
}

// AppUI is defined in ui/app.rs; config only holds configuration and shared data types.

// Cached state structs used by controllers/clients.
// Keep simple and cloneable for fast UI reads.
#[derive(Clone, Default, Debug)]
pub struct DuetState {
    pub connected: bool,
    pub last_error: Option<String>,
    pub status: Option<String>,
    pub last_command: Option<String>,
    pub position: [f32; 3],
}

#[derive(Clone, Default, Debug)]
pub struct MicrowaveState {
    pub connected: bool,
    pub enabled: bool,
    pub last_error: Option<String>,
    pub status: Option<String>,
    pub power_watts: f32,
    pub vswr: Option<f32>,
    pub forward_ratio: Option<f32>,
    pub temperature_c: Option<f32>,
}

#[derive(Clone, Default, Debug)]
pub struct ArduinoState {
    pub connected: bool,
    pub enabled: bool,
    pub pressure_setpoint_psi: f32,
    pub pressure_measured_psi: f32,
    pub loop_current_ma: Option<f32>,
    pub signal_ok: Option<bool>,
    pub status: Option<String>,
    pub last_error: Option<String>,
}
