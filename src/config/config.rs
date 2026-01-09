


pub const duet_ip: &str = "10.22.1.75";

pub const MICROWAVE_SERIAL_PORT: &str = "/dev/ttyUSB0";
pub const MICROWAVE_BAUD_RATE: u32 = 9600;


#[derive(Clone)]
pub struct DuetCommand {
    pub command: String,
    //pub axis: String,
    //pub value: f32,

    // FUTURE: stylistic implementation
    // 
}


#[derive(Clone)]
pub struct MicrowaveCommand {
    pub command: String,

}

use crate::controllers::{DuetController, MicrowaveController};

pub struct AppUI{
    pub duet: Box<dyn DuetController + Send + Sync>,
    pub microwave: Box<dyn MicrowaveController + Send + Sync>,
}

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
}