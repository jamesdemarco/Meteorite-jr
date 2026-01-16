use std::fmt;

/// A single step in a motion job.
/// Represents one row of movement commands.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub x_mm: f32,
    pub y_mm: f32,
    pub z_mm: f32,
    // Future columns can be added here (e.g., power_w, pressure_psi)
}

/// A motion job parsed from a CSV file.
/// Contains all steps and metadata about the job.
#[derive(Clone, Debug)]
pub struct Job {
    pub filename: String,
    pub steps: Vec<Step>,
    pub row_count: usize,        // number of parsed data rows (including header if counted)
    pub first_step: Option<Step>, // convenience preview of first step
    pub warnings: Vec<String>,   // warnings encountered during parsing (e.g., large file, many rows)
}

impl Job {
    /// Create a new Job from parsed steps.
    pub fn new(filename: String, steps: Vec<Step>) -> Self {
        Self::with_warnings(filename, steps, Vec::new())
    }

    /// Create a new Job from parsed steps with warnings.
    pub fn with_warnings(filename: String, steps: Vec<Step>, warnings: Vec<String>) -> Self {
        let row_count = steps.len();
        let first_step = steps.first().cloned();
        Self {
            filename,
            steps,
            row_count,
            first_step,
            warnings,
        }
    }
}

/// Errors that can occur when parsing a job CSV file.
#[derive(Clone, Debug)]
pub enum JobParseError {
    /// I/O error reading file
    Io(String),
    /// CSV parsing error
    Csv(String),
    /// Missing required header row
    MissingHeader(String),
    /// Missing required column in header
    MissingColumn(String),
    /// Failed to parse a number value
    BadNumber {
        column: String,
        row: usize,
        value: String,
    },
    /// Job has too many rows (exceeds limit)
    TooManyRows { max: usize, actual: usize },
    /// Job file is empty (no data rows)
    EmptyJob,
}

impl fmt::Display for JobParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobParseError::Io(msg) => write!(f, "I/O error: {}", msg),
            JobParseError::Csv(msg) => write!(f, "CSV error: {}", msg),
            JobParseError::MissingHeader(msg) => write!(f, "Missing header: {}", msg),
            JobParseError::MissingColumn(col) => write!(f, "Missing required column: {}", col),
            JobParseError::BadNumber {
                column,
                row,
                value,
            } => write!(
                f,
                "Failed to parse '{}' as number in column '{}' at row {}",
                value, column, row
            ),
            JobParseError::TooManyRows { max, actual } => {
                write!(f, "Too many rows: {} (max: {})", actual, max)
            }
            JobParseError::EmptyJob => write!(f, "Job file contains no data rows"),
        }
    }
}

impl std::error::Error for JobParseError {}
