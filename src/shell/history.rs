//! Command history management.
//!
//! Handles persistent command history with configurable limits.

use anyhow::Result;
use rustyline::config::Configurer;
use std::path::PathBuf;

/// Configuration for command history.
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    /// Path to the history file.
    pub path: PathBuf,
    /// Maximum number of entries to keep.
    pub max_entries: usize,
    /// Whether to ignore duplicate consecutive entries.
    #[allow(unused)]
    pub ignore_dups: bool,
    /// Whether to ignore entries starting with whitespace.
    #[allow(unused)]
    pub ignore_space: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("history"),
            max_entries: 1000,
            ignore_dups: true,
            ignore_space: true,
        }
    }
}

impl HistoryConfig {
    /// Creates a new history config with the given path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }

    /// Sets the maximum number of entries.
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Sets whether to ignore duplicate consecutive entries.
    #[allow(unused)]
    pub fn with_ignore_dups(mut self, ignore: bool) -> Self {
        self.ignore_dups = ignore;
        self
    }

    /// Sets whether to ignore entries starting with whitespace.
    #[allow(unused)]
    pub fn with_ignore_space(mut self, ignore: bool) -> Self {
        self.ignore_space = ignore;
        self
    }

    /// Applies this configuration to a rustyline DefaultEditor.
    #[allow(unused)]
    pub fn apply_to_default_editor(&self, editor: &mut rustyline::DefaultEditor) -> Result<()> {
        // Configure history behavior
        editor.set_max_history_size(self.max_entries)?;

        // Load existing history if the file exists
        if self.path.exists() {
            if let Err(e) = editor.load_history(&self.path) {
                log::warn!("Could not load history: {}", e);
            } else {
                log::debug!("Loaded history from {}", self.path.display());
            }
        }

        Ok(())
    }

    /// Saves history from a rustyline DefaultEditor.
    #[allow(unused)]
    pub fn save_from_default_editor(&self, editor: &mut rustyline::DefaultEditor) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        editor.save_history(&self.path)?;
        log::debug!("Saved history to {}", self.path.display());

        Ok(())
    }
}

/// Filters for determining what to add to history.
#[allow(unused)]
pub struct HistoryFilter {
    /// Configuration to use for filtering.
    config: HistoryConfig,
    /// Last entry added (for duplicate detection).
    last_entry: Option<String>,
}

impl HistoryFilter {
    /// Creates a new filter with the given config.
    #[allow(unused)]
    pub fn new(config: HistoryConfig) -> Self {
        Self {
            config,
            last_entry: None,
        }
    }

    /// Determines if an entry should be added to history.
    #[allow(unused)]
    pub fn should_add(&mut self, entry: &str) -> bool {
        let entry = entry.trim();

        // Ignore empty entries
        if entry.is_empty() {
            return false;
        }

        // Ignore entries starting with whitespace (if configured)
        if self.config.ignore_space && entry.starts_with(char::is_whitespace) {
            return false;
        }

        // Ignore duplicate consecutive entries (if configured)
        if self.config.ignore_dups {
            if let Some(ref last) = self.last_entry {
                if last == entry {
                    return false;
                }
            }
        }

        // Update last entry
        self.last_entry = Some(entry.to_string());
        true
    }

    /// Resets the filter state.
    #[allow(unused)]
    pub fn reset(&mut self) {
        self.last_entry = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_config_default() {
        let config = HistoryConfig::default();
        assert_eq!(config.max_entries, 1000);
        assert!(config.ignore_dups);
        assert!(config.ignore_space);
    }

    #[test]
    fn test_history_config_builder() {
        let config = HistoryConfig::new(PathBuf::from("/tmp/history"))
            .with_max_entries(500)
            .with_ignore_dups(false);

        assert_eq!(config.path, PathBuf::from("/tmp/history"));
        assert_eq!(config.max_entries, 500);
        assert!(!config.ignore_dups);
    }

    #[test]
    fn test_history_filter_empty() {
        let config = HistoryConfig::default();
        let mut filter = HistoryFilter::new(config);

        assert!(!filter.should_add(""));
        assert!(!filter.should_add("   "));
    }

    #[test]
    fn test_history_filter_duplicates() {
        let config = HistoryConfig::default();
        let mut filter = HistoryFilter::new(config);

        assert!(filter.should_add("add key value"));
        assert!(!filter.should_add("add key value")); // duplicate
        assert!(filter.should_add("get key")); // different
        assert!(filter.should_add("add key value")); // same as first, but not consecutive
    }

    #[test]
    fn test_history_filter_no_duplicate_check() {
        let config = HistoryConfig::default().with_ignore_dups(false);
        let mut filter = HistoryFilter::new(config);

        assert!(filter.should_add("add key value"));
        assert!(filter.should_add("add key value")); // allowed when ignore_dups is false
    }

    #[test]
    fn test_history_filter_reset() {
        let config = HistoryConfig::default();
        let mut filter = HistoryFilter::new(config);

        assert!(filter.should_add("command"));
        assert!(!filter.should_add("command")); // duplicate

        filter.reset();
        assert!(filter.should_add("command")); // allowed after reset
    }
}
