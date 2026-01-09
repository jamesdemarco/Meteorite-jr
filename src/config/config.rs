


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


pub struct MicrowaveCommand {
    pub command: String,

}

pub struct AppUI{
    pub duet_tx: tokio::sync::watch::Sender<DuetCommand>,
    pub microwave_tx: tokio::sync::watch::Sender<MicrowaveCommand>,
}