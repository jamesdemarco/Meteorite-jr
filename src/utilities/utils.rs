
/******************** Utilities Module ********************/

// holds utility functions for opening and polling connections to the Duet and Microwave devices






// imports

use tokio_serial::SerialStream;



//function to open duet connection (HTTP-based, URL helpers)
pub fn duet_base_url(duet_ip: &str) -> String {
    format!("http://{}", duet_ip)
}

pub fn rr_status_url(duet_ip: &str) -> String {
    format!("http://{}/rr_status?type=2", duet_ip)
}

pub fn rr_gcode_url(duet_ip: &str, gcode: &str) -> String {
    format!(
        "http://{}/rr_gcode?gcode={}",
        duet_ip,
        urlencoding::encode(gcode)
    )
}


// function to open microwave connection
pub async fn open_microwave_connection(
    microwave_serial_port_var: &str, 
    microwave_baud_rate: u32
) -> Result<SerialStream, Box<dyn std::error::Error + Send + Sync>> {

    let port_name = microwave_serial_port_var;
    let baud_rate = microwave_baud_rate;

    let builder = tokio_serial::new(port_name, baud_rate);
    let serial_stream = SerialStream::open(&builder)?;

    Ok(serial_stream)
}


// function to poll duet
pub async fn poll_duet() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Polling is now handled in the duet driver task via HTTP
    Ok(())
}