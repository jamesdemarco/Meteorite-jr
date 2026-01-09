/****************    DUET Control module   *************/

// handles communication with the DUET 2 board
// manages G-code sending and status receiving

// use reqwest

use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;
use crate::config::config::*;


pub async fn duet_control( 
    mut duet_connection: TcpStream, 
    mut duet_rx: watch::Receiver<DuetCommand> ) -> Result<(), Box<dyn std::error::Error>> {
    loop{
        

        // watch channel for Duet command

        if let Ok(command) = duet_rx.changed().await {
            let command = duet_rx.borrow().clone();
            // handle the command
            match command.command.as_str() {
                "G" => {
                    // send G-code command to Duet
                    duet_connection.write_all(command.command.as_bytes()).await?;
                },
                _ => {
                    // handle other commands
                }
            }
        }




        // poll Duet for updates/status

        // parse command if needed

        // send command to Duet board via network/ethernet

        


        
    }
}