/*!
# Logging Module

Structured logging system compatible with the Python PDW log format
while providing enhanced debugging capabilities.
*/

use crate::error::PdwError;
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

/// Initialize the logging system
pub fn init_logger(verbose: bool) -> Result<(), PdwError> {
    let log_level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    
    let mut builder = Builder::from_default_env();
    
    builder
        .target(Target::Stdout)
        .filter_level(log_level)
        .format(|buf, record| {
            let timestamp = chrono::Local::now().format("%Y/%m/%d %H:%M:%S");
            
            // Color coding for different log levels
            let level_color = match record.level() {
                log::Level::Error => "\x1b[31m", // Red
                log::Level::Warn => "\x1b[33m",  // Yellow
                log::Level::Info => "\x1b[32m",  // Green
                log::Level::Debug => "\x1b[36m", // Cyan
                log::Level::Trace => "\x1b[37m", // White
            };
            let reset_color = "\x1b[0m";
            
            writeln!(
                buf,
                "{} [{}{}{}] {}: {}",
                timestamp,
                level_color,
                record.level(),
                reset_color,
                record.target(),
                record.args()
            )
        })
        .init();
    
    Ok(())
}

/// Log processing step with consistent formatting
pub fn log_step(step_number: usize, description: &str, detail: &str) {
    log::info!(
        "   . .. ... Step: {:04} :-> {} :-> {}",
        step_number,
        description,
        detail
    );
}

/// Log processing result with count
pub fn log_result(description: &str, count: usize) {
    log::info!(
        "   . .. ... {} :-> \x1b[32m{:>6}\x1b[0m",
        description,
        count
    );
}

/// Log section separator (equivalent to Python's out_line)
pub fn log_separator() {
    log::info!("{}", "=".repeat(120));
}

/// Log processing phase start
pub fn log_phase_start(phase_name: &str) {
    log_separator();
    log::info!("{}", phase_name);
}

/// Log system information (equivalent to Python startup info)
pub fn log_system_info(
    version: &str,
    config_file: &str,
    yaml_file: &str,
    log_file: &str,
    input_file: &str,
    database_file: &str,
    guiding_sheet: &str,
) {
    log_separator();
    log::info!("Current Version         :-> \x1b[32m{}\x1b[0m", version);
    log::info!("Config/TOML File        :-> \x1b[32m{}\x1b[0m", config_file);
    log::info!("YAML Queries File       :-> \x1b[32m{}\x1b[0m", yaml_file);
    log::info!("LOG File                :-> \x1b[32m{}\x1b[0m", log_file);
    log::info!("Excel Sheet Input file  :-> \x1b[32m{}\x1b[0m", input_file);
    log::info!("Output SQLite3 Database :-> \x1b[32m{}\x1b[0m", database_file);
    log::info!("Guiding Excel Sheet     :-> \x1b[32m{}\x1b[0m", guiding_sheet);
    log_separator();
    log::info!("Personal Data Warehouse Processes are Starting | ET&L -> Extract, Transform & Loader !");
}

/// Log completion with timing information
pub fn log_completion(start_time: std::time::Instant, version: &str, hostname: &str) {
    let duration = start_time.elapsed();
    let total_seconds = duration.as_secs_f64();
    
    log_separator();
    log::info!("All Personal Data Warehouse processes have ended!");
    log::info!(
        "Processing completed in {:.2} seconds | Version {} | Hostname {} | OS {}",
        total_seconds,
        version,
        hostname,
        std::env::consts::OS
    );
    log_separator();
}

/// Create a file logger for persistent logging (equivalent to Python log file)
pub fn create_file_logger(log_file_path: &std::path::Path) -> Result<(), PdwError> {
    use std::fs::OpenOptions;
    
    // Ensure log directory exists
    if let Some(parent) = log_file_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            PdwError::Logging(format!("Failed to create log directory: {}", e))
        })?;
    }
    
    // Create or append to log file
    let _log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .map_err(|e| {
            PdwError::Logging(format!("Failed to open log file: {}", e))
        })?;
    
    Ok(())
}

/// Write completion entry to log file (equivalent to Python log_line)
pub fn write_log_entry(
    log_file_path: &std::path::Path,
    start_time: std::time::Instant,
    version: &str,
) -> Result<(), PdwError> {
    use std::fs::OpenOptions;
    use std::io::Write;
    
    let started = chrono::Local::now().format("%Y/%m/%d %H:%M:%S");
    let ended = chrono::Local::now().format("%Y/%m/%d %H:%M:%S");
    let duration = start_time.elapsed();
    let total_seconds = duration.as_secs_f64();
    let hostname = hostname::get()
        .unwrap_or_else(|_| "unknown".into())
        .to_string_lossy()
        .to_string();
    
    let log_entry = format!(
        "{} Started | {} Ended | {:.2} TotalSecs | Version {} | Hostname {} | OS {}\n",
        started,
        ended,
        total_seconds,
        version,
        hostname,
        std::env::consts::OS
    );
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .map_err(|e| {
            PdwError::Logging(format!("Failed to open log file for writing: {}", e))
        })?;
    
    file.write_all(log_entry.as_bytes()).map_err(|e| {
        PdwError::Logging(format!("Failed to write to log file: {}", e))
    })?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;
    
    #[test]
    fn test_logger_initialization() {
        let result = init_logger(false);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_file_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        
        let result = create_file_logger(&log_path);
        assert!(result.is_ok());
        assert!(log_path.exists());
    }
    
    #[test]
    fn test_log_entry_writing() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let start_time = std::time::Instant::now();
        
        let result = write_log_entry(&log_path, start_time, "9.11.0");
        assert!(result.is_ok());
        
        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("9.11.0"));
        assert!(content.contains("Started"));
        assert!(content.contains("Ended"));
    }
}