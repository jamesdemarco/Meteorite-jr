
/******************** Utilities Module ********************/

// holds utility functions for opening and polling connections to the Duet and Microwave devices






// imports

//use crate::config::config::*;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_serial::{SerialStream};



//function to open duet connection
pub async fn open_duet_connection(
    duet_ip_var: &str
) -> Result<TcpStream, Box<dyn std::error::Error>> {

    //let addr_str = duet_ip;

    // Try parsing as SocketAddr, otherwise append default port 23
    let socket_addr: SocketAddr = match duet_ip_var.parse() {
        Ok(a) => a,
        Err(_) => format!("{}:23", duet_ip_var).parse()?,
    };

    let stream = TcpStream::connect(socket_addr).await?;
    Ok(stream)
}


// function to open microwave connection
pub async fn open_microwave_connection(
    microwave_serial_port_var: &str, 
    microwave_baud_rate: u32
) -> Result<SerialStream, Box<dyn std::error::Error>> {

    let port_name = microwave_serial_port_var;
    let baud_rate = microwave_baud_rate;

    let builder = tokio_serial::new(port_name, baud_rate);
    let serial_stream = SerialStream::open(&builder)?;

    Ok(serial_stream)
}


// function to poll duet
pub async fn poll_duet(duet_connection: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {

    // probably adjust this to poll position 
    //let mut buffer = [0; 1024];
    //let n = duet_connection.read(&mut buffer).await?;
    //let response = String::from_utf8_lossy(&buffer[..n]).to_string();
    Ok(())
}