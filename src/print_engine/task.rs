use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};

use crate::controllers::{DuetController, MicrowaveController, ArduinoController};
use crate::job::Job;
use crate::print_engine::{PrintCommand, PrintState, PrintStatus};

pub async fn print_engine_task(
    mut rx: mpsc::Receiver<PrintCommand>,
    state: Arc<RwLock<PrintState>>,
    duet: Arc<Box<dyn DuetController + Send + Sync>>,
    microwave: Arc<Box<dyn MicrowaveController + Send + Sync>>,
    arduino: Arc<Box<dyn ArduinoController + Send + Sync>>,
    // access to UI-setpoints:
    microwave_power_setpoint: Arc<RwLock<f32>>,
    pressure_setpoint_psi: Arc<RwLock<f32>>,
) {
    let mut tick = interval(Duration::from_millis(20));
    
    // Current job and index tracking
    let mut current_job: Option<Arc<Job>> = None;
    let mut current_index: usize = 0;

    loop {
        tokio::select! {
            // Handle incoming commands
            Some(cmd) = rx.recv() => {
                match cmd {
                    PrintCommand::Start(job) => {
                        let total_steps = job.steps.len();
                        current_job = Some(job);
                        current_index = 0;
                        
                        // Idle -> Printing transition: controller side effects
                        // Set microwave power setpoint to 0
                        {
                            let mut mw_setpoint = microwave_power_setpoint.write().unwrap();
                            *mw_setpoint = 0.0;
                        }
                        microwave.set_power(0.0);
                        microwave.rf_on();
                        
                        // Enable arduino and set pressure
                        arduino.enable(true);
                        let pressure_sp = {
                            let sp = pressure_setpoint_psi.read().unwrap();
                            *sp
                        };
                        arduino.set_pressure_setpoint(pressure_sp);
                        
                        // Update state
                        let mut s = state.write().unwrap();
                        s.status = PrintStatus::Printing;
                        s.current_index = 0;
                        s.total_steps = total_steps;
                        s.last_error = None;
                    }
                    PrintCommand::Pause => {
                        let mut s = state.write().unwrap();
                        if s.status == PrintStatus::Printing {
                            s.status = PrintStatus::Paused;
                            drop(s); // Release lock before controller calls
                            
                            // Printing -> Paused transition: turn off controllers
                            microwave.rf_off();
                            arduino.set_pressure_setpoint(0.0);
                            arduino.enable(false);
                        }
                    }
                    PrintCommand::Resume => {
                        let mut s = state.write().unwrap();
                        if s.status == PrintStatus::Paused {
                            s.status = PrintStatus::Printing;
                            drop(s); // Release lock before controller calls
                            
                            // Paused -> Printing transition: turn on controllers
                            let mw_power = {
                                let sp = microwave_power_setpoint.read().unwrap();
                                *sp
                            };
                            microwave.set_power(mw_power);
                            microwave.rf_on();
                            
                            arduino.enable(true);
                            let pressure_sp = {
                                let sp = pressure_setpoint_psi.read().unwrap();
                                *sp
                            };
                            arduino.set_pressure_setpoint(pressure_sp);
                        }
                    }
                    PrintCommand::Abort => {
                        current_job = None;
                        current_index = 0;
                        
                        // Any -> Idle transition: turn off controllers (same as Pause)
                        microwave.rf_off();
                        arduino.set_pressure_setpoint(0.0);
                        arduino.enable(false);
                        
                        let mut s = state.write().unwrap();
                        s.status = PrintStatus::Idle;
                        s.current_index = 0;
                        s.total_steps = 0;
                        s.last_gcode = None;
                        // Keep last_error so user can see what happened
                    }
                }
            }
            
            // Process next step on tick if printing
            _ = tick.tick() => {
                let status = {
                    let s = state.read().unwrap();
                    s.status
                };
                
                if status == PrintStatus::Printing {
                    if let Some(ref job) = current_job {
                        if current_index < job.steps.len() {
                            let step = &job.steps[current_index];
                            
                            // Format G-code command
                            let gcode = format!(
                                "G1 X{:.3} Y{:.3} Z{:.3}",
                                step.x_mm, step.y_mm, step.z_mm
                            );
                            
                            // Send to duet (non-blocking enqueue)
                            duet.send_gcode(&gcode);
                            
                            // Update state
                            {
                                let mut s = state.write().unwrap();
                                s.current_index = current_index;
                                s.last_gcode = Some(gcode);
                            }
                            
                            current_index += 1;
                            
                            // Check if job is complete
                            if current_index >= job.steps.len() {
                                let mut s = state.write().unwrap();
                                s.status = PrintStatus::Idle;
                                s.current_index = current_index;
                                current_job = None;
                            }
                        }
                    }
                }
            }
        }
    }
}
