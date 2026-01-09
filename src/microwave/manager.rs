/****************    Microwave Control module   *************/

// handles commands to the microwave module

use tokio_serial::SerialStream;
use crate::config::config::*;   


pub async fn microwave_control(
    microwave: SerialStream, 
    microwave_rx: tokio::sync::watch::Receiver<MicrowaveCommand>
) -> Result<(), Box<dyn std::error::Error>> {
    loop{
        
        


        // watch channel for Microwave command


        // parse command if needed

        // send command to microwave via serial

        


        
    }
}