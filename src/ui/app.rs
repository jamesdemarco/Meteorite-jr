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
// In real mode, device tasks are spawned immediately but connect only on command
#[cfg(feature="real")] use tokio::sync::mpsc;
#[cfg(feature="real")] use std::sync::{Arc, RwLock};
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
    duet_pending: Option<PendingRequest>,
    microwave_pending: Option<PendingRequest>,
    // UI-only setpoints
    microwave_power_setpoint: f32,
    microwave_freq_setpoint: i32,
    duet_x_step: f32,
    duet_y_step: f32,
    duet_z_step: f32,
    duet_custom_gcode: String,
}

impl AppUI {
    pub fn new() -> Self {
        #[cfg(feature="mock")]
        {
            let duet: Box<dyn DuetController + Send + Sync> = Box::new(MockDuet::new());
            let microwave: Box<dyn MicrowaveController + Send + Sync> = Box::new(MockMicrowave::new());
            return Self {
                duet,
                microwave,
                duet_pending: None,
                microwave_pending: None,
                microwave_power_setpoint: 0.0,
                microwave_freq_setpoint: 0,
                duet_x_step: 0.0,
                duet_y_step: 0.0,
                duet_z_step: 0.0,
                duet_custom_gcode: String::new(),
            };
        }

        #[cfg(feature="real")]
        {
            // Shared state
            let duet_state = Arc::new(RwLock::new(DuetState::default()));
            let microwave_state = Arc::new(RwLock::new(MicrowaveState::default()));

            // Command channels (mpsc end-to-end)
            let (duet_cmd_tx, duet_cmd_rx) = mpsc::channel::<DuetCommand>(64);
            let (mw_cmd_tx, mw_cmd_rx) = mpsc::channel::<MicrowaveCommand>(64);

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

            let duet: Box<dyn DuetController + Send + Sync> = Box::new(DuetClient::new(duet_cmd_tx, duet_state));
            let microwave: Box<dyn MicrowaveController + Send + Sync> = Box::new(MicrowaveClient::new(mw_cmd_tx, microwave_state));
            return Self {
                duet,
                microwave,
                duet_pending: None,
                microwave_pending: None,
                microwave_power_setpoint: 0.0,
                microwave_freq_setpoint: 0,
                duet_x_step: 0.0,
                duet_y_step: 0.0,
                duet_z_step: 0.0,
                duet_custom_gcode: String::new(),
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
    fn send_microwave_set_frequency(&mut self, _hz: i32) {
        // TODO: implement frequency control in microwave controller
    }

    // Render microwave control section
    fn ui_center_microwave(&mut self, ui: &mut egui::Ui) {
        let microwave_state = self.microwave.state();

        ui.add_enabled_ui(microwave_state.connected, |ui| {
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
                                self.microwave_power_setpoint = (self.microwave_power_setpoint - 5.0).max(0.0);
                                if microwave_state.enabled {
                                    self.send_microwave_set_power(self.microwave_power_setpoint);
                                }
                            }
                            ui.add(egui::DragValue::new(&mut self.microwave_power_setpoint)
                                .speed(1.0)
                                .range(0.0..=f32::INFINITY));
                            if ui.button("+5").clicked() {
                                self.microwave_power_setpoint = (self.microwave_power_setpoint + 5.0).max(0.0);
                                if microwave_state.enabled {
                                    self.send_microwave_set_power(self.microwave_power_setpoint);
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

                    ui.add_space(10.0);

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
                                self.send_microwave_set_power(0.0);
                                } else {
                                self.send_microwave_set_power(self.microwave_power_setpoint);
                            }
                        }
                    });
                });
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
            // Microwave control section
            self.ui_center_microwave(ui);

            ui.add_space(12.0);

            // Duet control section
            self.ui_center_duet(ui);
        });
    }
}
