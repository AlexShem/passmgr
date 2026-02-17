//! Passmgr - A secure password manager library.
//!
//! This library provides the core functionality for the passmgr password manager,
//! including credential storage, encryption, and a shell-like interactive interface.

pub mod config;
pub mod credentials;
pub mod crypto;
pub mod logging;
pub mod manager;
pub mod shell;
pub mod storage;
pub mod trie;

// Re-export commonly used types
pub use config::AppConfig;
pub use credentials::Credentials;
pub use logging::{LogConfig, init_logging};
pub use manager::Manager;
pub use shell::Shell;
pub use trie::Trie;
