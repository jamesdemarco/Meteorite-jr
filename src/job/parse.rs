use std::path::Path;
use std::io::Read;
use csv::ReaderBuilder;

use super::model::{Job, JobParseError, Step};

// Threshold for large file warning (50 MB)
const LARGE_FILE_BYTES: u64 = 50 * 1024 * 1024;
// Threshold for many rows warning
const MANY_ROWS_THRESHOLD: usize = 100_000;

/// Load a motion job from a CSV file path.
/// 
/// # Arguments
/// * `path` - Path to the CSV file
/// * `max_rows` - Maximum number of data rows allowed (excludes header)
///
/// # Returns
/// * `Ok(Job)` - Successfully parsed job
/// * `Err(JobParseError)` - Parse error with details
///
/// # Format
/// CSV must have a header row with comma delimiter.
/// Required columns (case-insensitive, whitespace-trimmed):
/// - x or x_mm -> Step.x_mm
/// - y or y_mm -> Step.y_mm
/// - z or z_mm -> Step.z_mm
///
/// Row numbers in errors are 1-based data row indices (excluding header).
pub fn load_job_from_csv_path(path: &Path, max_rows: usize) -> Result<Job, JobParseError> {
    // Check file size before parsing
    let mut warnings = Vec::new();
    if let Ok(metadata) = std::fs::metadata(path) {
        let file_size = metadata.len();
        if file_size > LARGE_FILE_BYTES {
            warnings.push(format!(
                "Large file: {} MB (threshold: {} MB)",
                file_size / (1024 * 1024),
                LARGE_FILE_BYTES / (1024 * 1024)
            ));
        }
    }
    
    // Read file
    let file = std::fs::File::open(path)
        .map_err(|e| JobParseError::Io(format!("Failed to open file: {}", e)))?;
    
    // Get filename
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.csv".to_string());
    
    // Parse from reader
    load_job_from_csv_reader(file, &filename, max_rows, warnings)
}

/// Load a motion job from a CSV reader.
/// Internal helper that can be used for testing with in-memory data.
/// 
/// # Arguments
/// * `reader` - Any type implementing Read (file, cursor, etc.)
/// * `filename` - Name to assign to the job
/// * `max_rows` - Maximum number of data rows allowed (excludes header)
/// * `warnings` - Pre-existing warnings to include (e.g., from file size check)
fn load_job_from_csv_reader<R: Read>(
    reader: R,
    filename: &str,
    max_rows: usize,
    mut warnings: Vec<String>,
) -> Result<Job, JobParseError> {
    // Build CSV reader with headers
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(reader);
    
    // Get headers
    let headers = reader.headers()
        .map_err(|e| JobParseError::Csv(format!("Failed to read headers: {}", e)))?;
    
    if headers.is_empty() {
        return Err(JobParseError::MissingHeader("CSV file has no header row".to_string()));
    }
    
    // Find column indices for x, y, z (case-insensitive, trimmed)
    let x_idx = find_column_index(headers, &["x", "x_mm"])
        .ok_or_else(|| JobParseError::MissingColumn("x or x_mm".to_string()))?;
    let y_idx = find_column_index(headers, &["y", "y_mm"])
        .ok_or_else(|| JobParseError::MissingColumn("y or y_mm".to_string()))?;
    let z_idx = find_column_index(headers, &["z", "z_mm"])
        .ok_or_else(|| JobParseError::MissingColumn("z or z_mm".to_string()))?;
    
    // Parse data rows
    let mut steps = Vec::new();
    let mut data_row = 0; // 1-based data row index (excludes header)
    
    for result in reader.records() {
        data_row += 1;
        
        let record = result.map_err(|e| JobParseError::Csv(format!("Row {}: {}", data_row, e)))?;
        
        // Check max_rows before parsing
        if data_row > max_rows {
            return Err(JobParseError::TooManyRows {
                max: max_rows,
                actual: data_row,
            });
        }
        
        // Parse x, y, z values
        let x_mm = parse_float(&record, x_idx, "x", data_row)?;
        let y_mm = parse_float(&record, y_idx, "y", data_row)?;
        let z_mm = parse_float(&record, z_idx, "z", data_row)?;
        
        steps.push(Step { x_mm, y_mm, z_mm });
    }
    
    // Check if empty
    if steps.is_empty() {
        return Err(JobParseError::EmptyJob);
    }
    
    // Check for many rows
    if steps.len() > MANY_ROWS_THRESHOLD {
        warnings.push(format!(
            "Many rows: {} (threshold: {})",
            steps.len(),
            MANY_ROWS_THRESHOLD
        ));
    }
    
    // Build Job
    Ok(Job::with_warnings(filename.to_string(), steps, warnings))
}

/// Find the index of a column matching one of the given names (case-insensitive, trimmed).
fn find_column_index(headers: &csv::StringRecord, names: &[&str]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let header_lower = header.trim().to_lowercase();
        for name in names {
            if header_lower == name.to_lowercase() {
                return Some(idx);
            }
        }
    }
    None
}

/// Parse a float value from a record at the given index.
fn parse_float(
    record: &csv::StringRecord,
    idx: usize,
    column_name: &str,
    row: usize,
) -> Result<f32, JobParseError> {
    let value = record.get(idx).ok_or_else(|| {
        JobParseError::Csv(format!("Row {}: missing column index {}", row, idx))
    })?;
    
    let trimmed = value.trim();
    trimmed.parse::<f32>().map_err(|_| JobParseError::BadNumber {
        column: column_name.to_string(),
        row,
        value: trimmed.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Helper to parse CSV from a string
    fn parse_csv(csv_data: &str, max_rows: usize) -> Result<Job, JobParseError> {
        let cursor = Cursor::new(csv_data.as_bytes());
        load_job_from_csv_reader(cursor, "test.csv", max_rows, Vec::new())
    }

    #[test]
    fn test_accepts_short_headers() {
        let csv = "x,y,z\n1.0,2.0,3.0\n4.5,5.5,6.5";
        let job = parse_csv(csv, 1000).unwrap();
        
        assert_eq!(job.filename, "test.csv");
        assert_eq!(job.steps.len(), 2);
        assert_eq!(job.row_count, 2);
        assert_eq!(job.steps[0].x_mm, 1.0);
        assert_eq!(job.steps[0].y_mm, 2.0);
        assert_eq!(job.steps[0].z_mm, 3.0);
        assert_eq!(job.steps[1].x_mm, 4.5);
        assert_eq!(job.steps[1].y_mm, 5.5);
        assert_eq!(job.steps[1].z_mm, 6.5);
        assert!(job.first_step.is_some());
        assert_eq!(job.first_step.as_ref().unwrap().x_mm, 1.0);
    }

    #[test]
    fn test_accepts_long_headers() {
        let csv = "x_mm,y_mm,z_mm\n10.0,20.0,30.0";
        let job = parse_csv(csv, 1000).unwrap();
        
        assert_eq!(job.steps.len(), 1);
        assert_eq!(job.steps[0].x_mm, 10.0);
        assert_eq!(job.steps[0].y_mm, 20.0);
        assert_eq!(job.steps[0].z_mm, 30.0);
    }

    #[test]
    fn test_case_insensitive_headers() {
        let csv = "X,Y,Z\n1.0,2.0,3.0";
        let job = parse_csv(csv, 1000).unwrap();
        
        assert_eq!(job.steps.len(), 1);
        assert_eq!(job.steps[0].x_mm, 1.0);
    }

    #[test]
    fn test_whitespace_in_headers() {
        let csv = " x , y , z \n1.0,2.0,3.0";
        let job = parse_csv(csv, 1000).unwrap();
        
        assert_eq!(job.steps.len(), 1);
        assert_eq!(job.steps[0].x_mm, 1.0);
    }

    #[test]
    fn test_missing_column_x() {
        let csv = "y,z\n2.0,3.0";
        let result = parse_csv(csv, 1000);
        
        assert!(matches!(result, Err(JobParseError::MissingColumn(ref col)) if col.contains("x")));
    }

    #[test]
    fn test_missing_column_y() {
        let csv = "x,z\n1.0,3.0";
        let result = parse_csv(csv, 1000);
        
        assert!(matches!(result, Err(JobParseError::MissingColumn(ref col)) if col.contains("y")));
    }

    #[test]
    fn test_missing_column_z() {
        let csv = "x,y\n1.0,2.0";
        let result = parse_csv(csv, 1000);
        
        assert!(matches!(result, Err(JobParseError::MissingColumn(ref col)) if col.contains("z")));
    }

    #[test]
    fn test_bad_number_with_details() {
        let csv = "x,y,z\n1.0,abc,3.0";
        let result = parse_csv(csv, 1000);
        
        match result {
            Err(JobParseError::BadNumber { column, row, value }) => {
                assert_eq!(column, "y");
                assert_eq!(row, 1); // First data row
                assert_eq!(value, "abc");
            }
            _ => panic!("Expected BadNumber error, got {:?}", result),
        }
    }

    #[test]
    fn test_bad_number_second_row() {
        let csv = "x,y,z\n1.0,2.0,3.0\n4.0,5.0,invalid";
        let result = parse_csv(csv, 1000);
        
        match result {
            Err(JobParseError::BadNumber { column, row, value }) => {
                assert_eq!(column, "z");
                assert_eq!(row, 2); // Second data row
                assert_eq!(value, "invalid");
            }
            _ => panic!("Expected BadNumber error, got {:?}", result),
        }
    }

    #[test]
    fn test_enforces_max_rows() {
        let csv = "x,y,z\n1.0,2.0,3.0\n4.0,5.0,6.0\n7.0,8.0,9.0";
        let result = parse_csv(csv, 2);
        
        match result {
            Err(JobParseError::TooManyRows { max, actual }) => {
                assert_eq!(max, 2);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected TooManyRows error, got {:?}", result),
        }
    }

    #[test]
    fn test_empty_job() {
        let csv = "x,y,z\n";
        let result = parse_csv(csv, 1000);
        
        assert!(matches!(result, Err(JobParseError::EmptyJob)));
    }

    #[test]
    fn test_empty_job_no_data_rows() {
        let csv = "x,y,z";
        let result = parse_csv(csv, 1000);
        
        assert!(matches!(result, Err(JobParseError::EmptyJob)));
    }

    #[test]
    fn test_negative_values() {
        let csv = "x,y,z\n-1.5,-2.5,-3.5";
        let job = parse_csv(csv, 1000).unwrap();
        
        assert_eq!(job.steps[0].x_mm, -1.5);
        assert_eq!(job.steps[0].y_mm, -2.5);
        assert_eq!(job.steps[0].z_mm, -3.5);
    }

    #[test]
    fn test_decimal_precision() {
        let csv = "x,y,z\n1.123456,2.789012,3.456789";
        let job = parse_csv(csv, 1000).unwrap();
        
        // Note: f32 has limited precision
        assert!((job.steps[0].x_mm - 1.123456).abs() < 0.0001);
    }

    #[test]
    fn test_mixed_header_formats() {
        let csv = "x,y_mm,Z\n1.0,2.0,3.0";
        let job = parse_csv(csv, 1000).unwrap();
        
        assert_eq!(job.steps.len(), 1);
        assert_eq!(job.steps[0].x_mm, 1.0);
        assert_eq!(job.steps[0].y_mm, 2.0);
        assert_eq!(job.steps[0].z_mm, 3.0);
    }

    #[test]
    fn test_extra_columns_ignored() {
        let csv = "x,y,z,extra,columns\n1.0,2.0,3.0,4.0,5.0";
        let job = parse_csv(csv, 1000).unwrap();
        
        assert_eq!(job.steps.len(), 1);
        assert_eq!(job.steps[0].x_mm, 1.0);
        assert_eq!(job.steps[0].y_mm, 2.0);
        assert_eq!(job.steps[0].z_mm, 3.0);
    }

    #[test]
    fn test_warnings_empty_by_default() {
        let csv = "x,y,z\n1.0,2.0,3.0";
        let job = parse_csv(csv, 1000).unwrap();
        
        assert!(job.warnings.is_empty());
    }
}
