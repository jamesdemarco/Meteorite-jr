

/*************** Program Entry Point *******************/

// 
// spawns single threaded async tasks for each module
// handles shutdown of tasks 

// module declaration
mod ui;
mod utilities;
mod duet;
mod microwave;
mod config;



// imports
use crate::config::config::*;
use crate::utilities::utils::{open_duet_connection, open_microwave_connection};
use tokio::sync::watch;


//use tokio::sync::mpsc;
//use tokio_serial::{SerialStream};
//use tokio::net::tcp;


// main function
#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let duet = open_duet_connection(duet_ip).await?;
    let microwave = open_microwave_connection(MICROWAVE_SERIAL_PORT, MICROWAVE_BAUD_RATE).await?;
    let (duet_tx, duet_rx) = watch::channel(DuetCommand{command: String::from("G")});
    let (microwave_tx, microwave_rx) = watch::channel(MicrowaveCommand{command: String::from("S")});



    // Cached state containers
    use std::sync::{Arc, RwLock};
    let duet_state = Arc::new(RwLock::new(DuetState::default()));
    let microwave_state = Arc::new(RwLock::new(MicrowaveState::default()));

    // Duet task
    let duet_state_task = Arc::clone(&duet_state);
    tokio::spawn(async move{
        if let Err(e) = duet::manager::duet_control(duet, duet_rx, duet_state_task).await {
            eprintln!("error in Duet task: {:?}", e);
        }
    });

    // Microwave task
    let microwave_state_task = Arc::clone(&microwave_state);
    tokio::spawn(async move{
        if let Err(e) = microwave::manager::microwave_control(microwave, microwave_rx, microwave_state_task).await {
            eprintln!("error in Microwave task: {:?}", e);
        }
    });


    // start UI
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "AppUI",
        options,
        Box::new(|_cc| 
            Ok(Box::new(AppUI::new(
                duet_tx.clone(),
                microwave_tx.clone(),
            )))
        ),
    )?;
    tokio::signal::ctrl_c().await?;
    Ok(())

}



    


