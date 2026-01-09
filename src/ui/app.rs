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
use crate::controllers::{DuetController, MicrowaveController};
#[cfg(feature="mock")] use crate::controllers::duet::MockDuet;
#[cfg(feature="mock")] use crate::controllers::microwave::MockMicrowave;
#[cfg(feature="real")] use crate::controllers::duet::DuetClient;
#[cfg(feature="real")] use crate::controllers::microwave::MicrowaveClient;
#[cfg(feature="real")] use crate::utilities::utils::{open_duet_connection, open_microwave_connection};
#[cfg(feature="real")] use tokio::sync::{mpsc, watch};
#[cfg(feature="real")] use std::sync::{Arc, RwLock};
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
    pub fn new() -> Self {
        #[cfg(feature="mock")]
        {
            let duet: Box<dyn DuetController + Send + Sync> = Box::new(MockDuet::new());
            let microwave: Box<dyn MicrowaveController + Send + Sync> = Box::new(MockMicrowave::new());
            return Self { duet, microwave };
        }

        #[cfg(feature="real")]
        {
            // Shared state
            let duet_state = Arc::new(RwLock::new(DuetState::default()));
            let microwave_state = Arc::new(RwLock::new(MicrowaveState::default()));

            // Command channels (client side)
            let (duet_cmd_tx, mut duet_cmd_rx) = mpsc::channel::<DuetCommand>(64);
            let (mw_cmd_tx, mut mw_cmd_rx) = mpsc::channel::<MicrowaveCommand>(64);

            // Device task channels (watch)
            let (duet_tx, duet_rx) = watch::channel(DuetCommand{ command: String::from("INIT") });
            let (microwave_tx, microwave_rx) = watch::channel(MicrowaveCommand{ command: String::from("INIT") });

            // Forwarder tasks from mpsc -> watch
            tokio::spawn({
                let duet_tx = duet_tx.clone();
                async move {
                    while let Some(cmd) = duet_cmd_rx.recv().await {
                        let _ = duet_tx.send(cmd);
                    }
                }
            });
            tokio::spawn({
                let microwave_tx = microwave_tx.clone();
                async move {
                    while let Some(cmd) = mw_cmd_rx.recv().await {
                        let _ = microwave_tx.send(cmd);
                    }
                }
            });

            // Connect devices and spawn control tasks
            tokio::spawn({
                let state_for_task = Arc::clone(&duet_state);
                let state_for_err = Arc::clone(&duet_state);
                async move {
                    match open_duet_connection(duet_ip).await {
                        Ok(duet_conn) => {
                            if let Err(e) = crate::duet::manager::duet_control(duet_conn, duet_rx, state_for_task).await {
                                let mut s = state_for_err.write().unwrap();
                                s.last_error = Some(format!("Duet task error: {}", e));
                                s.connected = false;
                                s.status = Some("error".into());
                            }
                        }
                        Err(e) => {
                            let err = e.to_string();
                            let mut s = state_for_err.write().unwrap();
                            s.last_error = Some(format!("Connect failed: {}", err));
                            s.connected = false;
                            s.status = Some("disconnected".into());
                        }
                    }
                }
            });

            tokio::spawn({
                let state_for_task = Arc::clone(&microwave_state);
                let state_for_err = Arc::clone(&microwave_state);
                async move {
                    match open_microwave_connection(MICROWAVE_SERIAL_PORT, MICROWAVE_BAUD_RATE).await {
                        Ok(mw_conn) => {
                            if let Err(e) = crate::microwave::manager::microwave_control(mw_conn, microwave_rx, state_for_task).await {
                                let mut s = state_for_err.write().unwrap();
                                s.last_error = Some(format!("Microwave task error: {}", e));
                                s.connected = false;
                                s.status = Some("error".into());
                            }
                        }
                        Err(e) => {
                            let err = e.to_string();
                            let mut s = state_for_err.write().unwrap();
                            s.last_error = Some(format!("Connect failed: {}", err));
                            s.connected = false;
                            s.status = Some("disconnected".into());
                        }
                    }
                }
            });

            let duet: Box<dyn DuetController + Send + Sync> = Box::new(DuetClient::new(duet_cmd_tx, duet_state));
            let microwave: Box<dyn MicrowaveController + Send + Sync> = Box::new(MicrowaveClient::new(mw_cmd_tx, microwave_state));
            return Self { duet, microwave };
        }
    }
}


impl eframe::App for AppUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Meteorite Jr Control Panel");

            if ui.button("Send Duet Command G28").clicked() {
                self.duet.send_gcode("G28");
            }

            if ui.button("Set Microwave Power 10W").clicked() {
                self.microwave.set_power(10.0);
            }

            // Minimal state display
            let d = self.duet.state();
            ui.label(format!("Duet: connected={} status={:?} pos=({:.1},{:.1},{:.1})", d.connected, d.status, d.position[0], d.position[1], d.position[2]));
            let m = self.microwave.state();
            ui.label(format!("Microwave: connected={} status={:?} power={:.1}W", m.connected, m.status, m.power_watts));
        });
    }
}