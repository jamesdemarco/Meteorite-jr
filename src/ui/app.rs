/************** UI Module  ********************/

// builds the front end GUI for control purposes
/*  builds a static egui SidePanel UI for display / control:
        - jog control
            - 
            - 
            - 

        - microwave control
            - 

        - pressure control
            - 

        - 
            - 

        - 
            - 
            
        - 
            - 
            - 
            - 
            - 

*/ 




// imports

use crate::config::config::*;
use eframe::egui;
//use egui_plot::Legend;
//use tokio::sync::mpsc;
//use tokio::sync::watch;
//use std::sync::Arc;
//use tokio::sync::Mutex;
//use egui_plot::{Plot, Line};
//use std::time::Instant;
//use csv::Writer;
//use chrono::Local;
//use std::fs; //added for local file path
//use std::env; //added for local file path
//use std::error::Error; //added for local file path
//use std::path::PathBuf; //added for local file path



impl AppUI {
    pub fn new(
        duet_tx: tokio::sync::watch::Sender<DuetCommand>,
        microwave_tx: tokio::sync::watch::Sender<MicrowaveCommand>,
    ) -> Self {
        Self {
            duet_tx,
            microwave_tx,
        }
    }
}


impl eframe::App for AppUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Meteorite Jr Control Panel");

            if ui.button("Send Duet Command G28").clicked() {
                let cmd = DuetCommand {
                    command: String::from("G28"),
                };
                let _ = self.duet_tx.send(cmd);
            }

            if ui.button("Send Microwave Command S10").clicked() {
                let cmd = MicrowaveCommand {
                    command: String::from("S10"),
                };
                let _ = self.microwave_tx.send(cmd);
            }
        });
    }
}