use std::sync::Arc;

use crate::job::Job;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintStatus {
    Idle,
    Printing,
    Paused,
}

#[derive(Debug)]
pub enum PrintCommand {
    Start(Arc<Job>),
    Pause,
    Resume,
    Abort,
}

#[derive(Debug, Clone)]
pub struct PrintState {
    pub status: PrintStatus,
    pub current_index: usize,     // index into job.steps
    pub total_steps: usize,       // job.steps.len()
    pub last_gcode: Option<String>,
    pub last_error: Option<String>,
}

impl Default for PrintState {
    fn default() -> Self {
        Self {
            status: PrintStatus::Idle,
            current_index: 0,
            total_steps: 0,
            last_gcode: None,
            last_error: None,
        }
    }
}
