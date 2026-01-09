

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
mod controllers;



// imports
use crate::config::config::*;
use crate::ui::app::AppUI;
// wiring is centralized in AppUI::new via feature flags


//use tokio::sync::mpsc;
//use tokio_serial::{SerialStream};
//use tokio::net::tcp;


// main function
#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Device tasks and channels are set up by AppUI::new()


    // start UI
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "AppUI",
        options,
        Box::new(|_cc| 
            Ok(Box::new(AppUI::new()))
        ),
    )?;
    tokio::signal::ctrl_c().await?;
    Ok(())

}



    


