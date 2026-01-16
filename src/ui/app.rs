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
use crate::controllers::{DuetController, MicrowaveController, ArduinoController};
#[cfg(feature="mock")] use crate::controllers::duet::MockDuet;
#[cfg(feature="mock")] use crate::controllers::microwave::MockMicrowave;
#[cfg(feature="mock")] use crate::controllers::arduino::MockArduino;
#[cfg(feature="real")] use crate::controllers::duet::DuetClient;
#[cfg(feature="real")] use crate::controllers::microwave::MicrowaveClient;
#[cfg(feature="real")] use crate::controllers::arduino::ArduinoClient;
// In real mode, device tasks are spawned immediately but connect only on command
use tokio::sync::mpsc;
use std::sync::{Arc, RwLock};
use crate::print_engine::{PrintCommand, PrintState, print_engine_task};
use eframe::egui;
use std::time::Instant;
//use egui_plot::Legend;
//use tokio::sync::mpsc;
//use tokio::sync::watch;
//use std::sync::Arc;
//use tokio::sync::Mutex;
//use egui_plot::{Plot, Line};
//use csv::Writer;
//use chrono::Local;
//use std::fs; //added for local file path
//use std::env; //added for local file path
//use std::error::Error; //added for local file path
//use std::path::PathBuf; //added for local file path


// Pending action tracking for connection requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingAction { Connect, Disconnect }

#[derive(Debug, Clone)]
struct PendingRequest {
    action: PendingAction,
    started_at: Instant,
}

// App-level type holding controller trait objects.
pub struct AppUI{
    pub duet: Box<dyn DuetController + Send + Sync>,
    pub microwave: Box<dyn MicrowaveController + Send + Sync>,
    pub arduino: Box<dyn ArduinoController + Send + Sync>,
    duet_pending: Option<PendingRequest>,
    microwave_pending: Option<PendingRequest>,
    arduino_pending: Option<PendingRequest>,
    // Print engine
    print_cmd_tx: mpsc::Sender<PrintCommand>,
    print_state: Arc<RwLock<PrintState>>,
    // Shared setpoints (used by both UI and print engine)
    microwave_power_setpoint: Arc<RwLock<f32>>,
    microwave_freq_setpoint: i32,
    arduino_pressure_setpoint: Arc<RwLock<f32>>,
    duet_x_step: f32,
    duet_y_step: f32,
    duet_z_step: f32,
    duet_custom_gcode: String,
    // Toolpath creation fields
    toolpath_file_name: String,
    toolpath_parse_error: String,
    toolpath_row_count: usize,
    toolpath_start_x: f32,
    toolpath_start_y: f32,
    toolpath_start_z: f32,
    current_job: Option<std::sync::Arc<crate::job::Job>>,
}

impl AppUI {
    pub fn new() -> Self {
        #[cfg(feature="mock")]
        {
            let duet: Box<dyn DuetController + Send + Sync> = Box::new(MockDuet::new());
            let microwave: Box<dyn MicrowaveController + Send + Sync> = Box::new(MockMicrowave::new());
            let arduino: Box<dyn ArduinoController + Send + Sync> = Box::new(MockArduino::new());
            
            // Print engine setup
            let (print_cmd_tx, print_cmd_rx) = mpsc::channel::<PrintCommand>(64);
            let print_state = Arc::new(RwLock::new(PrintState::default()));
            let microwave_power_setpoint = Arc::new(RwLock::new(0.0f32));
            let arduino_pressure_setpoint = Arc::new(RwLock::new(0.0f32));
            
            // Controllers for print engine (wrap boxes in Arc)
            let duet_arc: Arc<Box<dyn DuetController + Send + Sync>> = Arc::new(Box::new(MockDuet::new()));
            let microwave_arc: Arc<Box<dyn MicrowaveController + Send + Sync>> = Arc::new(Box::new(MockMicrowave::new()));
            let arduino_arc: Arc<Box<dyn ArduinoController + Send + Sync>> = Arc::new(Box::new(MockArduino::new()));
            
            // Spawn print engine task
            tokio::spawn(print_engine_task(
                print_cmd_rx,
                Arc::clone(&print_state),
                duet_arc,
                microwave_arc,
                arduino_arc,
                Arc::clone(&microwave_power_setpoint),
                Arc::clone(&arduino_pressure_setpoint),
            ));
            
            return Self {
                duet,
                microwave,
                arduino,
                duet_pending: None,
                microwave_pending: None,
                arduino_pending: None,
                print_cmd_tx,
                print_state,
                microwave_power_setpoint,
                microwave_freq_setpoint: 0,
                arduino_pressure_setpoint,
                duet_x_step: 0.0,
                duet_y_step: 0.0,
                duet_z_step: 0.0,
                duet_custom_gcode: String::new(),
                toolpath_file_name: String::new(),
                toolpath_parse_error: String::new(),
                toolpath_row_count: 0,
                toolpath_start_x: 0.0,
                toolpath_start_y: 0.0,
                toolpath_start_z: 0.0,
                current_job: None,
            };
        }

        #[cfg(feature="real")]
        {
            // Shared state
            let duet_state = Arc::new(RwLock::new(DuetState::default()));
            let microwave_state = Arc::new(RwLock::new(MicrowaveState::default()));
            let arduino_state = Arc::new(RwLock::new(ArduinoState::default()));

            // Command channels (mpsc end-to-end)
            let (duet_cmd_tx, duet_cmd_rx) = mpsc::channel::<DuetCommand>(64);
            let (mw_cmd_tx, mw_cmd_rx) = mpsc::channel::<MicrowaveCommand>(64);
            let (arduino_cmd_tx, arduino_cmd_rx) = mpsc::channel::<ArduinoCommand>(64);

            // Duet device task. Connect only on command.
            tokio::spawn({
                let state_for_task = Arc::clone(&duet_state);
                async move {
                    let _ = crate::drivers::duet::duet_control(duet_ip, duet_cmd_rx, state_for_task).await;
                }
            });

            // Microwave device task. Connect only on command.
            tokio::spawn({
                let state_for_task = Arc::clone(&microwave_state);
                async move {
                    let _ = crate::drivers::microwave::microwave_control(MICROWAVE_SERIAL_PORT, MICROWAVE_BAUD_RATE, mw_cmd_rx, state_for_task).await;
                }
            });

            // Arduino device task. Connect only on command.
            tokio::spawn({
                let state_for_task = Arc::clone(&arduino_state);
                async move {
                    let _ = crate::drivers::arduino::task::arduino_control(ARDUINO_SERIAL_PORT, ARDUINO_BAUD_RATE, arduino_cmd_rx, state_for_task).await;
                }
            });

            let duet: Box<dyn DuetController + Send + Sync> = Box::new(DuetClient::new(duet_cmd_tx.clone(), Arc::clone(&duet_state)));
            let microwave: Box<dyn MicrowaveController + Send + Sync> = Box::new(MicrowaveClient::new(mw_cmd_tx.clone(), Arc::clone(&microwave_state)));
            let arduino: Box<dyn ArduinoController + Send + Sync> = Box::new(ArduinoClient::new(arduino_cmd_tx.clone(), Arc::clone(&arduino_state)));
            
            // Print engine setup
            let (print_cmd_tx, print_cmd_rx) = mpsc::channel::<PrintCommand>(64);
            let print_state = Arc::new(RwLock::new(PrintState::default()));
            let microwave_power_setpoint = Arc::new(RwLock::new(0.0f32));
            let arduino_pressure_setpoint = Arc::new(RwLock::new(0.0f32));
            
            // Controllers for print engine (wrap boxes in Arc)
            let duet_arc: Arc<Box<dyn DuetController + Send + Sync>> = Arc::new(Box::new(DuetClient::new(duet_cmd_tx, Arc::clone(&duet_state))));
            let microwave_arc: Arc<Box<dyn MicrowaveController + Send + Sync>> = Arc::new(Box::new(MicrowaveClient::new(mw_cmd_tx, Arc::clone(&microwave_state))));
            let arduino_arc: Arc<Box<dyn ArduinoController + Send + Sync>> = Arc::new(Box::new(ArduinoClient::new(arduino_cmd_tx, Arc::clone(&arduino_state))));
            
            // Spawn print engine task
            tokio::spawn(print_engine_task(
                print_cmd_rx,
                Arc::clone(&print_state),
                duet_arc,
                microwave_arc,
                arduino_arc,
                Arc::clone(&microwave_power_setpoint),
                Arc::clone(&arduino_pressure_setpoint),
            ));
            
            return Self {
                duet,
                microwave,
                arduino,
                duet_pending: None,
                microwave_pending: None,
                arduino_pending: None,
                print_cmd_tx,
                print_state,
                microwave_power_setpoint,
                microwave_freq_setpoint: 0,
                arduino_pressure_setpoint,
                duet_x_step: 0.0,
                duet_y_step: 0.0,
                duet_z_step: 0.0,
                duet_custom_gcode: String::new(),
                toolpath_file_name: String::new(),
                toolpath_parse_error: String::new(),
                toolpath_row_count: 0,
                toolpath_start_x: 0.0,
                toolpath_start_y: 0.0,
                toolpath_start_z: 0.0,
                current_job: None,
            };
        }
    }

    // Stub method for sending duet gcode
    fn send_duet_gcode(&mut self, gcode: String) {
        // TODO: call duet controller SendGcodeCommand
        self.duet.send_gcode(&gcode);
    }

    // Stub method for sending microwave power command
    fn send_microwave_set_power(&mut self, watts: f32) {
        // TODO: call microwave controller
        self.microwave.set_power(watts);
    }

    // Stub method for sending microwave frequency command
    fn send_microwave_set_frequency(&mut self, hz: i32) {
        self.microwave.set_frequency(hz);
    }

    // Render microwave control section
    fn ui_center_microwave(&mut self, ui: &mut egui::Ui) {
        let microwave_state = self.microwave.state();
        let panel_h = 165.0;

        ui.add_enabled_ui(microwave_state.connected, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(),panel_h),
                egui::Layout::top_down(egui::Align::Min),
        |ui| {
            egui::Frame::group(ui.style())
                .show(ui, |ui| {
                    ui.heading("Microwave Control");
                    ui.add_space(5.0);

                    egui::Grid::new("microwave_grid")
                        .num_columns(4)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            // Power row
                            ui.label("Power (W)");
                            if ui.button("-5").clicked() {
                                let new_val = {
                                    let mut sp = self.microwave_power_setpoint.write().unwrap();
                                    *sp = (*sp - 5.0).max(0.0);
                                    *sp
                                };
                                if microwave_state.enabled {
                                    self.send_microwave_set_power(new_val);
                                }
                            }
                            let mut power_val = self.microwave_power_setpoint.read().unwrap().clone();
                            if ui.add(egui::DragValue::new(&mut power_val)
                                .speed(1.0)
                                .range(0.0..=f32::INFINITY)).changed() {
                                *self.microwave_power_setpoint.write().unwrap() = power_val;
                            }
                            if ui.button("+5").clicked() {
                                let new_val = {
                                    let mut sp = self.microwave_power_setpoint.write().unwrap();
                                    *sp = (*sp + 5.0).max(0.0);
                                    *sp
                                };
                                if microwave_state.enabled {
                                    self.send_microwave_set_power(new_val);
                                }
                            }
                            ui.end_row();

                            // Frequency row
                            ui.label("Frequency (Hz)");
                            if ui.button("-5").clicked() {
                                self.microwave_freq_setpoint = (self.microwave_freq_setpoint - 5).max(0);
                                if microwave_state.enabled {
                                    self.send_microwave_set_frequency(self.microwave_freq_setpoint);
                                }
                            }
                            ui.add(egui::DragValue::new(&mut self.microwave_freq_setpoint)
                                .speed(1)
                                .range(0..=i32::MAX));
                            if ui.button("+5").clicked() {
                                self.microwave_freq_setpoint = (self.microwave_freq_setpoint + 5).max(0);
                                if microwave_state.enabled {
                                    self.send_microwave_set_frequency(self.microwave_freq_setpoint);
                                }
                            }
                            ui.end_row();
                        });

                    //ui.add_space(10.0);

                    // ON/OFF button
                    // ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    //     let button_label = if microwave_state.enabled { "ON" } else { "OFF" };
                    //     let button = egui::Button::new(button_label)
                    //         .min_size(egui::vec2(120.0, 40.0));
                    //     if ui.add(button).clicked() {
                    //         if microwave_state.enabled {
                    //             // Turn OFF
                    //             self.send_microwave_set_power(0.0);
                    //         } else {
                    //             // Turn ON with current setpoint
                    //             self.send_microwave_set_power(self.microwave_power_setpoint);
                    //         }
                    //     }
                    // });
                    ui.add_space(8.0);
                    ui.horizontal_centered(|ui| {
                        let button_label = if microwave_state.enabled { "ON" } else { "OFF" };
                        let button = egui::Button::new(button_label);
                        if ui.add_sized([200.0, 40.0], button).clicked() {
                            if microwave_state.enabled {
                                // Turn OFF
                                self.microwave.rf_off();
                            } else {
                                // Turn ON with current setpoint
                                let power_sp = *self.microwave_power_setpoint.read().unwrap();
                                self.microwave.set_power(power_sp);
                                self.microwave.rf_on();
                            }
                        }
                    });
                });
            },
        );
    });
}

    // Render pressure control section
    fn ui_center_pressure(&mut self, ui: &mut egui::Ui) {
        let arduino_state = self.arduino.state();
        let panel_w = 360.0;
        let panel_h = 165.0;

        ui.add_enabled_ui(arduino_state.connected, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(panel_w, panel_h),
                egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::Frame::group(ui.style())
                    .show(ui, |ui| {
                        ui.heading("Pressure Control");
                        ui.add_space(5.0);

                        egui::Grid::new("pressure_grid")
                            .num_columns(4)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                // Pressure row
                                ui.label("Pressure (psi)");
                                if ui.button("-1").clicked() {
                                    let new_val = {
                                        let mut sp = self.arduino_pressure_setpoint.write().unwrap();
                                        *sp = (*sp - 1.0).max(0.0);
                                        *sp
                                    };
                                    if arduino_state.enabled {
                                        self.arduino.set_pressure_setpoint(new_val);
                                    }
                                }
                                let mut pressure_val = self.arduino_pressure_setpoint.read().unwrap().clone();
                                if ui.add(egui::DragValue::new(&mut pressure_val)
                                    .speed(0.1)
                                    .range(0.0..=f32::INFINITY)).changed() {
                                    *self.arduino_pressure_setpoint.write().unwrap() = pressure_val;
                                }
                                if ui.button("+1").clicked() {
                                    let new_val = {
                                        let mut sp = self.arduino_pressure_setpoint.write().unwrap();
                                        *sp = (*sp + 1.0).max(0.0);
                                        *sp
                                    };
                                    if arduino_state.enabled {
                                        self.arduino.set_pressure_setpoint(new_val);
                                    }
                                }
                                ui.end_row();

                                // Row 2: current pressure readback (gauge)
                                ui.label("Current Pressure:");
                                ui.label(format!("{:.1} psi", arduino_state.pressure_measured_psi));
                                ui.label(""); // spacer to fill columns 3-4
                                ui.label("");
                                ui.end_row();
                            });

                        ui.add_space(8.0);
                        ui.horizontal_centered(|ui| {
                            let button_label = if arduino_state.enabled { "ON" } else { "OFF" };
                            let button = egui::Button::new(button_label);
                            if ui.add_sized([200.0, 40.0], button).clicked() {
                                if arduino_state.enabled {
                                    // Turn OFF
                                    self.arduino.set_pressure_setpoint(0.0);
                                    self.arduino.enable(false);
                                } else {
                                    // Turn ON with current setpoint
                                    let pressure_sp = *self.arduino_pressure_setpoint.read().unwrap();
                                    self.arduino.set_pressure_setpoint(pressure_sp);
                                    self.arduino.enable(true);
                                }
                            }
                        });
                    });
            },
        );
    });
}

    // Render duet control section (jog + send)
    fn ui_center_duet(&mut self, ui: &mut egui::Ui) {
        let duet_state = self.duet.state();

        ui.add_enabled_ui(duet_state.connected, |ui| {
            egui::Frame::group(ui.style())
                .show(ui, |ui| {
                    ui.heading("Duet Control");
                    ui.add_space(5.0);

                    let button_size = egui::vec2(40.0, 28.0);
                    let send_button_size = egui::vec2(64.0, 28.0);

                    egui::Grid::new("duet_grid")
                        .num_columns(9)
                        .spacing([6.0, 6.0])
                        .show(ui, |ui| {
                            // X row
                            ui.label("X");
                            for delta in [10.0, 5.0, 1.0, -1.0, -5.0, -10.0].iter() {
                                let btn = egui::Button::new(format!("{:+}", delta)).min_size(button_size);
                                if ui.add(btn).clicked() {
                                    let gcode = format!("G1 X{:+.3}", delta);
                                    self.send_duet_gcode(gcode);
                                }
                            }
                            ui.add(
                                egui::DragValue::new(&mut self.duet_x_step)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX)
                                    .min_decimals(0)
                                    .max_decimals(3)
                                    .suffix(" mm"),
                            );
                            let send_btn = egui::Button::new("Send").min_size(send_button_size);
                            if ui.add(send_btn).clicked() {
                                let gcode = format!("G1 X{:+.3}", self.duet_x_step);
                                self.send_duet_gcode(gcode);
                            }
                            ui.end_row();

                            // Y row
                            ui.label("Y");
                            for delta in [10.0, 5.0, 1.0, -1.0, -5.0, -10.0].iter() {
                                let btn = egui::Button::new(format!("{:+}", delta)).min_size(button_size);
                                if ui.add(btn).clicked() {
                                    let gcode = format!("G1 Y{:+.3}", delta);
                                    self.send_duet_gcode(gcode);
                                }
                            }
                            ui.add(
                                egui::DragValue::new(&mut self.duet_y_step)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX)
                                    .min_decimals(0)
                                    .max_decimals(3)
                                    .suffix(" mm"),
                            );
                            let send_btn = egui::Button::new("Send").min_size(send_button_size);
                            if ui.add(send_btn).clicked() {
                                let gcode = format!("G1 Y{:+.3}", self.duet_y_step);
                                self.send_duet_gcode(gcode);
                            }
                            ui.end_row();

                            // Z row
                            ui.label("Z");
                            for delta in [10.0, 5.0, 1.0, -1.0, -5.0, -10.0].iter() {
                                let btn = egui::Button::new(format!("{:+}", delta)).min_size(button_size);
                                if ui.add(btn).clicked() {
                                    let gcode = format!("G1 Z{:+.3}", delta);
                                    self.send_duet_gcode(gcode);
                                }
                            }
                            ui.add(
                                egui::DragValue::new(&mut self.duet_z_step)
                                    .speed(0.5)
                                    .range(f32::MIN..=f32::MAX)
                                    .min_decimals(0)
                                    .max_decimals(3)
                                    .suffix(" mm"),
                            );
                            let send_btn = egui::Button::new("Send").min_size(send_button_size);
                            if ui.add(send_btn).clicked() {
                                let gcode = format!("G1 Z{:+.3}", self.duet_z_step);
                                self.send_duet_gcode(gcode);
                            }
                            ui.end_row();
                        });

                    ui.add_space(10.0);

                    // Custom send row
                    ui.horizontal(|ui| {
                        ui.label("Send Custom");
                        ui.add_sized(
                            egui::vec2(220.0, 24.0),
                            egui::TextEdit::singleline(&mut self.duet_custom_gcode),
                        );
                        if ui.button("Send Custom").clicked() {
                            let cmd = self.duet_custom_gcode.trim_end_matches('\n').to_string();
                            if !cmd.is_empty() {
                                self.send_duet_gcode(cmd);
                            }
                        }
                    });
                });
        });
    }

    // Render toolpath creation section
    fn ui_toolpath_creation(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .show(ui, |ui| {
                ui.heading("Toolpath Creation");
                ui.add_space(5.0);

                egui::Grid::new("toolpath_grid")
                    .num_columns(3)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        // Row 1: Upload button, filename display, Clear button
                        if ui.button("Upload File").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("CSV", &["csv"])
                                .pick_file()
                            {
                                match crate::job::load_job_from_csv_path(&path, 1000) {
                                    Ok(job) => {
                                        self.toolpath_file_name = job.filename.clone();
                                        self.toolpath_row_count = job.row_count;
                                        if let Some(step) = job.first_step.clone() {
                                            self.toolpath_start_x = step.x_mm;
                                            self.toolpath_start_y = step.y_mm;
                                            self.toolpath_start_z = step.z_mm;
                                        } else {
                                            self.toolpath_start_x = 0.0;
                                            self.toolpath_start_y = 0.0;
                                            self.toolpath_start_z = 0.0;
                                        }
                                        self.toolpath_parse_error.clear();
                                        self.current_job = Some(std::sync::Arc::new(job));
                                    }
                                    Err(e) => {
                                        self.toolpath_parse_error = e.to_string();
                                        self.toolpath_file_name.clear();
                                        self.toolpath_row_count = 0;
                                        self.toolpath_start_x = 0.0;
                                        self.toolpath_start_y = 0.0;
                                        self.toolpath_start_z = 0.0;
                                        self.current_job = None;
                                    }
                                }
                            }
                        }

                        let mut filename_display = self.toolpath_file_name.clone();
                        ui.add_sized(
                            [(ui.available_width() - 100.0).max(100.0), 24.0],
                            egui::TextEdit::singleline(&mut filename_display).interactive(false),
                        );

                        if ui.button("Clear").clicked() {
                            self.toolpath_file_name.clear();
                            self.toolpath_parse_error.clear();
                            self.toolpath_row_count = 0;
                            self.toolpath_start_x = 0.0;
                            self.toolpath_start_y = 0.0;
                            self.toolpath_start_z = 0.0;
                            self.current_job = None;
                        }
                        ui.end_row();

                        // Row 2: Parse errors
                        ui.label("Parse Errors:");
                        let mut error_display = self.toolpath_parse_error.clone();
                        ui.add_sized(
                            [(ui.available_width()).max(100.0), 24.0],
                            egui::TextEdit::singleline(&mut error_display).interactive(false),
                        );
                        ui.label(""); // Empty cell for alignment
                        ui.end_row();

                        // Row 3: Starting Position
                        ui.label("Starting Position:");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            let mut x_display = format!("{:.3}", self.toolpath_start_x);
                            ui.add_sized(
                                [80.0, 24.0],
                                egui::TextEdit::singleline(&mut x_display).interactive(false),
                            );
                            ui.label("Y:");
                            let mut y_display = format!("{:.3}", self.toolpath_start_y);
                            ui.add_sized(
                                [80.0, 24.0],
                                egui::TextEdit::singleline(&mut y_display).interactive(false),
                            );
                            ui.label("Z:");
                            let mut z_display = format!("{:.3}", self.toolpath_start_z);
                            ui.add_sized(
                                [80.0, 24.0],
                                egui::TextEdit::singleline(&mut z_display).interactive(false),
                            );
                        });
                        ui.label(""); // Empty cell for alignment
                        ui.end_row();

                        // Row 4: Row Count
                        ui.label("Row Count:");
                        let mut count_display = format!("{}", self.toolpath_row_count);
                        ui.add_sized(
                            [150.0, 24.0],
                            egui::TextEdit::singleline(&mut count_display).interactive(false),
                        );
                        ui.label(""); // Empty cell for alignment
                        ui.end_row();
                    });
            });
    }

    // Render print controls section
    fn ui_print_controls(&mut self, ui: &mut egui::Ui) {
        // Panel enabled only if job loaded and duet connected
        let has_job = self.current_job.is_some();
        let duet_connected = self.duet.state().connected;
        let panel_enabled = has_job && duet_connected;

        ui.add_enabled_ui(panel_enabled, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.heading("Print Controls");
                ui.add_space(8.0);

                // Read current print state
                let ps = self.print_state.read().unwrap().clone();

                // Button enable states based on PrintStatus
                let (start_enabled, pause_enabled, resume_enabled, abort_enabled) = match ps.status {
                    crate::print_engine::PrintStatus::Idle => (true, false, false, false),
                    crate::print_engine::PrintStatus::Printing => (false, true, false, true),
                    crate::print_engine::PrintStatus::Paused => (false, false, true, true),
                };

                // Row of control buttons
                ui.horizontal(|ui| {
                    // Start button
                    if ui.add_enabled(start_enabled, egui::Button::new("Start")).clicked() {
                        if let Some(ref job) = self.current_job {
                            let cmd = crate::print_engine::PrintCommand::Start(Arc::clone(job));
                            if let Err(e) = self.print_cmd_tx.try_send(cmd) {
                                // Could set error in state, but for now just log
                                eprintln!("Failed to send Start command: {}", e);
                            }
                        }
                    }

                    // Pause button
                    if ui.add_enabled(pause_enabled, egui::Button::new("Pause")).clicked() {
                        let cmd = crate::print_engine::PrintCommand::Pause;
                        if let Err(e) = self.print_cmd_tx.try_send(cmd) {
                            eprintln!("Failed to send Pause command: {}", e);
                        }
                    }

                    // Resume button
                    if ui.add_enabled(resume_enabled, egui::Button::new("Resume")).clicked() {
                        let cmd = crate::print_engine::PrintCommand::Resume;
                        if let Err(e) = self.print_cmd_tx.try_send(cmd) {
                            eprintln!("Failed to send Resume command: {}", e);
                        }
                    }

                    // Abort button
                    if ui.add_enabled(abort_enabled, egui::Button::new("Abort")).clicked() {
                        let cmd = crate::print_engine::PrintCommand::Abort;
                        if let Err(e) = self.print_cmd_tx.try_send(cmd) {
                            eprintln!("Failed to send Abort command: {}", e);
                        }
                    }
                });

                ui.add_space(8.0);

                // Status display
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    let last_gcode_display = ps.last_gcode.as_deref().unwrap_or("—");
                    let status_text = format!(
                        "{:?}  step {}/{}  last: {}",
                        ps.status, ps.current_index, ps.total_steps, last_gcode_display
                    );
                    ui.label(status_text);
                });

                // Display last error if present
                if let Some(ref error) = ps.last_error {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Error:");
                        ui.colored_label(egui::Color32::RED, error);
                    });
                }
            });
        });
    }

    // Update pending request state, clearing when confirmed or timed out (8 seconds)
    fn update_pending_requests(&mut self) {
        const TIMEOUT_SECS: u64 = 8;
        let now = Instant::now();

        // Duet pending
        if let Some(ref pending) = self.duet_pending {
            let elapsed = now.duration_since(pending.started_at).as_secs();
            let state = self.duet.state();
            let should_clear = match pending.action {
                PendingAction::Connect => state.connected || elapsed > TIMEOUT_SECS,
                PendingAction::Disconnect => !state.connected || elapsed > TIMEOUT_SECS,
            };
            if should_clear {
                self.duet_pending = None;
            }
        }

        // Microwave pending
        if let Some(ref pending) = self.microwave_pending {
            let elapsed = now.duration_since(pending.started_at).as_secs();
            let state = self.microwave.state();
            let should_clear = match pending.action {
                PendingAction::Connect => state.connected || elapsed > TIMEOUT_SECS,
                PendingAction::Disconnect => !state.connected || elapsed > TIMEOUT_SECS,
            };
            if should_clear {
                self.microwave_pending = None;
            }
        }

        // Arduino pending
        if let Some(ref pending) = self.arduino_pending {
            let elapsed = now.duration_since(pending.started_at).as_secs();
            let state = self.arduino.state();
            let should_clear = match pending.action {
                PendingAction::Connect => state.connected || elapsed > TIMEOUT_SECS,
                PendingAction::Disconnect => !state.connected || elapsed > TIMEOUT_SECS,
            };
            if should_clear {
                self.arduino_pending = None;
            }
        }
    }

    // Render left panel with connection controls
    fn ui_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Connections");
                ui.add_space(10.0);

                // Duet section
                ui.group(|ui| {
                    ui.label("Duet");
                    let duet_state = self.duet.state();
                    let (button_label, button_enabled) = if let Some(ref pending) = self.duet_pending {
                        match pending.action {
                            PendingAction::Connect => ("Connecting…", false),
                            PendingAction::Disconnect => ("Disconnecting…", false),
                        }
                    } else if duet_state.connected {
                        ("Disconnect", true)
                    } else {
                        ("Connect", true)
                    };

                    let button = egui::Button::new(button_label)
                        .min_size(egui::vec2(ui.available_width(), 44.0));
                    if ui.add_enabled(button_enabled, button).clicked() {
                        if duet_state.connected {
                            self.duet.disconnect();
                            self.duet_pending = Some(PendingRequest {
                                action: PendingAction::Disconnect,
                                started_at: Instant::now(),
                            });
                        } else {
                            self.duet.connect();
                            self.duet_pending = Some(PendingRequest {
                                action: PendingAction::Connect,
                                started_at: Instant::now(),
                            });
                        }
                    }

                    if let Some(ref status) = duet_state.status {
                        ui.label(status);
                    }
                    if let Some(ref err) = duet_state.last_error {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                    }
                });

                ui.add_space(10.0);

                // Microwave section
                ui.group(|ui| {
                    ui.label("Microwave");
                    let microwave_state = self.microwave.state();
                    let (button_label, button_enabled) = if let Some(ref pending) = self.microwave_pending {
                        match pending.action {
                            PendingAction::Connect => ("Connecting…", false),
                            PendingAction::Disconnect => ("Disconnecting…", false),
                        }
                    } else if microwave_state.connected {
                        ("Disconnect", true)
                    } else {
                        ("Connect", true)
                    };

                    let button = egui::Button::new(button_label)
                        .min_size(egui::vec2(ui.available_width(), 44.0));
                    if ui.add_enabled(button_enabled, button).clicked() {
                        if microwave_state.connected {
                            self.microwave.disconnect();
                            self.microwave_pending = Some(PendingRequest {
                                action: PendingAction::Disconnect,
                                started_at: Instant::now(),
                            });
                        } else {
                            self.microwave.connect();
                            self.microwave_pending = Some(PendingRequest {
                                action: PendingAction::Connect,
                                started_at: Instant::now(),
                            });
                        }
                    }

                    if let Some(ref status) = microwave_state.status {
                        ui.label(status);
                    }
                    if let Some(ref err) = microwave_state.last_error {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                    }
                });

                ui.add_space(10.0);

                // Arduino section
                ui.group(|ui| {
                    ui.label("Arduino (Pressure)");
                    let arduino_state = self.arduino.state();
                    let (button_label, button_enabled) = if let Some(ref pending) = self.arduino_pending {
                        match pending.action {
                            PendingAction::Connect => ("Connecting…", false),
                            PendingAction::Disconnect => ("Disconnecting…", false),
                        }
                    } else if arduino_state.connected {
                        ("Disconnect", true)
                    } else {
                        ("Connect", true)
                    };

                    let button = egui::Button::new(button_label)
                        .min_size(egui::vec2(ui.available_width(), 44.0));
                    if ui.add_enabled(button_enabled, button).clicked() {
                        if arduino_state.connected {
                            self.arduino.disconnect();
                            self.arduino_pending = Some(PendingRequest {
                                action: PendingAction::Disconnect,
                                started_at: Instant::now(),
                            });
                        } else {
                            self.arduino.connect();
                            self.arduino_pending = Some(PendingRequest {
                                action: PendingAction::Connect,
                                started_at: Instant::now(),
                            });
                        }
                    }

                    if let Some(ref status) = arduino_state.status {
                        ui.label(status);
                    }
                    if let Some(ref err) = arduino_state.last_error {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                    }
                });
            });
    }

    // Render right panel with telemetry displays
    fn ui_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Telemetry");
                ui.add_space(5.0);

                egui::Grid::new("telemetry_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        // Duet section
                        let duet_state = self.duet.state();

                        ui.label("Duet Connected");
                        ui.label(if duet_state.connected { "Yes" } else { "No" });
                        ui.end_row();

                        ui.label("Duet Status");
                        ui.label(duet_state.status.as_deref().unwrap_or("—"));
                        ui.end_row();

                        ui.label("Last Command");
                        ui.label(duet_state.last_command.as_deref().unwrap_or("—"));
                        ui.end_row();

                        ui.label("X Position");
                        ui.label(format!("{:.1} mm", duet_state.position[0]));
                        ui.end_row();

                        ui.label("Y Position");
                        ui.label(format!("{:.1} mm", duet_state.position[1]));
                        ui.end_row();

                        ui.label("Z Position");
                        ui.label(format!("{:.1} mm", duet_state.position[2]));
                        ui.end_row();

                        // Spacing row
                        ui.label("");
                        ui.label("");
                        ui.end_row();

                        // Microwave section
                        let microwave_state = self.microwave.state();

                        ui.label("Microwave Connected");
                        ui.label(if microwave_state.connected { "Yes" } else { "No" });
                        ui.end_row();

                        ui.label("Microwave Enabled");
                        ui.label(if microwave_state.enabled { "Yes" } else { "No" });
                        ui.end_row();

                        ui.label("Microwave Status");
                        ui.label(microwave_state.status.as_deref().unwrap_or("—"));
                        ui.end_row();

                        ui.label("Power");
                        ui.label(format!("{:.1} W", microwave_state.power_watts));
                        ui.end_row();

                        ui.label("VSWR");
                        let vswr_text = microwave_state.vswr
                            .map(|v| format!("{:.1}", v))
                            .unwrap_or_else(|| "—".to_string());
                        ui.label(vswr_text);
                        ui.end_row();

                        ui.label("Forward Ratio");
                        let fr_text = microwave_state.forward_ratio
                            .map(|v| format!("{:.1}", v))
                            .unwrap_or_else(|| "—".to_string());
                        ui.label(fr_text);
                        ui.end_row();

                        ui.label("Temperature");
                        let temp_text = microwave_state.temperature_c
                            .map(|t| format!("{:.1} °C", t))
                            .unwrap_or_else(|| "—".to_string());
                        ui.label(temp_text);
                        ui.end_row();

                        // Spacing row
                        ui.label("");
                        ui.label("");
                        ui.end_row();

                        // Pressure (placeholder)
                        ui.label("Pressure");
                        ui.label("—");
                        ui.end_row();
                    });
            });
    }
}


impl eframe::App for AppUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update pending request state each frame
        self.update_pending_requests();

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                    ui.heading("TB1 Control Panel");
                },
            );
        });

        // Left panel with connection controls
        self.ui_left_panel(ctx);

        // Right panel with telemetry
        self.ui_right_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            // Top row: Microwave and Pressure control sections side-by-side
            ui.horizontal(|ui| {
                // Left: Microwave control
                self.ui_center_microwave(ui);
                ui.add_space(12.0);
                // Right: Pressure control
                self.ui_center_pressure(ui);
            });

            ui.add_space(12.0);

            // Bottom: Duet control section
            self.ui_center_duet(ui);

            ui.add_space(12.0);

            // Print Controls section
            self.ui_print_controls(ui);

            ui.add_space(12.0);

            // Toolpath creation section
            self.ui_toolpath_creation(ui);
        });
    }
}
