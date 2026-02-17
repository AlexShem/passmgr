//! Configuration and path management for passmgr.
//!
//! This module handles all file paths used by the application,
//! including the password database, command history, and log files.

use anyhow::{Result, anyhow};
use std::path::PathBuf;

/// The name of the application directory.
const APP_DIR: &str = ".passmgr";

/// Default history file name.
const HISTORY_FILE: &str = "history";

/// Default log file name.
const LOG_FILE: &str = "passmgr.log";

/// Default password database file name.
const DB_FILE: &str = "passwords.db";

/// Maximum number of history entries to keep.
pub const DEFAULT_HISTORY_SIZE: usize = 1000;

/// Returns the base directory for passmgr data (~/.passmgr).
///
/// Creates the directory if it doesn't exist.
pub fn get_app_dir() -> Result<PathBuf> {
    let home_path =
        dirs_next::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;

    let app_dir = home_path.join(APP_DIR);

    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }

    Ok(app_dir)
}

/// Returns the path to the password database file.
///
/// The database is stored at `~/.passmgr/passwords.db`.
/// Creates the parent directory and an empty file if they don't exist.
pub fn get_password_db() -> Result<PathBuf> {
    let app_dir = get_app_dir()?;
    let db_path = app_dir.join(DB_FILE);

    if !db_path.exists() {
        std::fs::File::create(&db_path)?;
    }

    Ok(db_path)
}

/// Returns the path to the command history file.
///
/// The history is stored at `~/.passmgr/history`.
/// Creates the parent directory if it doesn't exist.
pub fn get_history_path() -> Result<PathBuf> {
    let app_dir = get_app_dir()?;
    Ok(app_dir.join(HISTORY_FILE))
}

/// Returns the path to the log file.
///
/// The log is stored at `~/.passmgr/passmgr.log`.
/// Creates the parent directory if it doesn't exist.
pub fn get_log_path() -> Result<PathBuf> {
    let app_dir = get_app_dir()?;
    Ok(app_dir.join(LOG_FILE))
}

/// Application configuration loaded from environment or defaults.
#[derive(Debug, Clone)]
#[allow(unused)]
pub struct AppConfig {
    /// Path to the password database.
    pub db_path: PathBuf,
    /// Path to the command history file.
    pub history_path: PathBuf,
    /// Path to the log file.
    pub log_path: PathBuf,
    /// Maximum number of history entries.
    pub history_size: usize,
}

impl AppConfig {
    /// Loads configuration with default paths.
    ///
    /// All paths are relative to `~/.passmgr/`.
    #[allow(unused)]
    pub fn load() -> Result<Self> {
        Ok(Self {
            db_path: get_password_db()?,
            history_path: get_history_path()?,
            log_path: get_log_path()?,
            history_size: DEFAULT_HISTORY_SIZE,
        })
    }

    /// Creates a configuration for testing with custom base directory.
    #[cfg(test)]
    pub fn for_testing(base_dir: &std::path::Path) -> Self {
        Self {
            db_path: base_dir.join(DB_FILE),
            history_path: base_dir.join(HISTORY_FILE),
            log_path: base_dir.join(LOG_FILE),
            history_size: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_app_dir() {
        // This test requires a home directory to be set
        if dirs_next::home_dir().is_some() {
            let result = get_app_dir();
            assert!(result.is_ok());
            let path = result.unwrap();
            assert!(path.ends_with(APP_DIR));
        }
    }

    #[test]
    fn test_app_config_for_testing() {
        let temp_dir = TempDir::new().unwrap();
        let config = AppConfig::for_testing(temp_dir.path());

        assert_eq!(config.db_path, temp_dir.path().join(DB_FILE));
        assert_eq!(config.history_path, temp_dir.path().join(HISTORY_FILE));
        assert_eq!(config.log_path, temp_dir.path().join(LOG_FILE));
        assert_eq!(config.history_size, 100);
    }

    #[test]
    fn test_default_history_size() {
        assert_eq!(DEFAULT_HISTORY_SIZE, 1000);
    }
}
