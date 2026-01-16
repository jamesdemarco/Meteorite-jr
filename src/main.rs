

/*************** Program Entry Point *******************/

// 
// spawns single threaded async tasks for each module
// handles shutdown of tasks 

// module declaration
mod ui;
mod utilities;
mod drivers;
mod config;
mod controllers;
mod job;
mod print_engine;



// imports
use crate::config::config::*;
use crate::ui::app::AppUI;
// wiring is centralized in AppUI::new via feature flags
use eframe::egui;

//use tokio::sync::mpsc;
//use tokio_serial::{SerialStream};
//use tokio::net::tcp;


// main function
#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Device tasks and channels are set up by AppUI::new()


    // start UI
    let options = eframe::NativeOptions{
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
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



    


