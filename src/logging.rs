//! Logging infrastructure for passmgr.
//!
//! This module provides structured logging with file output, timestamps,
//! and configurable log levels.

use anyhow::{Result, anyhow};
use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::OpenOptions;
use std::path::PathBuf;

/// Configuration for the logging system.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Path to the log file.
    pub path: PathBuf,
    /// Minimum log level to record.
    pub level: LevelFilter,
    /// Maximum log file size in bytes before rotation (0 = no limit).
    pub max_size: u64,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("passmgr.log"),
            level: LevelFilter::Info,
            max_size: 10 * 1024 * 1024, // 10 MB
        }
    }
}

impl LogConfig {
    /// Creates a new LogConfig with the specified path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }

    /// Sets the log level.
    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// Sets the maximum log file size.
    pub fn with_max_size(mut self, max_size: u64) -> Self {
        self.max_size = max_size;
        self
    }
}

/// Initializes the logging system with the given configuration.
///
/// This sets up a combined logger that writes to both:
/// - Terminal (with colors, at Info level or higher for user feedback)
/// - Log file (at the configured level, with timestamps)
///
/// # Example
///
/// ```ignore
/// use passmgr::logging::{init_logging, LogConfig};
/// use log::LevelFilter;
///
/// let config = LogConfig::new("~/.passmgr/passmgr.log".into())
///     .with_level(LevelFilter::Trace);
///
/// init_logging(&config)?;
/// log::info!("Application started");
/// ```
pub fn init_logging(config: &LogConfig) -> Result<()> {
    // Ensure the parent directory exists
    if let Some(parent) = config.path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Check if we need to rotate the log file
    if config.max_size > 0 && config.path.exists() {
        if let Ok(metadata) = std::fs::metadata(&config.path) {
            if metadata.len() > config.max_size {
                rotate_log(&config.path)?;
            }
        }
    }

    // Open or create the log file
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.path)
        .map_err(|e| anyhow!("Failed to open log file: {}", e))?;

    // Build logger configuration with timestamps
    let file_config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Debug)
        .build();

    let term_config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off) // No timestamps in terminal
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![];

    // File logger at configured level
    loggers.push(WriteLogger::new(config.level, file_config, log_file));

    // Terminal logger at Warn level (only important messages)
    // Only add terminal logger if we're running in a terminal
    if atty_check() {
        loggers.push(TermLogger::new(
            LevelFilter::Warn,
            term_config,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ));
    }

    CombinedLogger::init(loggers).map_err(|e| anyhow!("Failed to initialize logger: {}", e))?;

    log::info!("Logging initialized at level {:?}", config.level);
    log::debug!("Log file: {}", config.path.display());

    Ok(())
}

/// Simple check if stdout is a TTY.
fn atty_check() -> bool {
    // We'll use a simple heuristic - try to detect if we're in a terminal
    std::env::var("TERM").is_ok()
}

/// Rotates the log file by renaming it with a timestamp suffix.
fn rotate_log(path: &PathBuf) -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let rotated_name = format!(
        "{}.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("passmgr.log"),
        timestamp
    );

    let rotated_path = path.with_file_name(rotated_name);
    std::fs::rename(path, &rotated_path)?;

    log::info!("Rotated log file to: {}", rotated_path.display());
    Ok(())
}

/// Log a timed operation.
///
/// Returns the result of the operation and logs the duration.
#[allow(unused)]
pub fn timed<T, F: FnOnce() -> T>(operation: &str, f: F) -> T {
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();

    log::debug!("{} completed in {:?}", operation, duration);
    result
}

/// Macro for logging operations with timing.
#[macro_export]
macro_rules! log_timed {
    ($op:expr, $body:expr) => {{
        let start = std::time::Instant::now();
        let result = $body;
        let duration = start.elapsed();
        log::debug!("{} completed in {:?}", $op, duration);
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, LevelFilter::Info);
        assert_eq!(config.max_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_log_config_builder() {
        let config = LogConfig::new(PathBuf::from("/tmp/test.log"))
            .with_level(LevelFilter::Trace)
            .with_max_size(1024);

        assert_eq!(config.path, PathBuf::from("/tmp/test.log"));
        assert_eq!(config.level, LevelFilter::Trace);
        assert_eq!(config.max_size, 1024);
    }

    #[test]
    fn test_timed_operation() {
        let result = timed("test operation", || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        assert_eq!(result, 42);
    }
}
