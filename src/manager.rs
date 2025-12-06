//! Password manager core functionality.
//!
//! This module handles credential management, encryption, and persistence.

use anyhow::{Result, anyhow};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::config::{DEFAULT_HISTORY_SIZE, get_history_path};
use crate::credentials::Credentials;
use crate::crypto::{decrypt, derive_key, encrypt, generate_nonce, generate_salt};
use crate::shell::history::HistoryConfig;
use crate::shell::{Shell, ShellConfig};
use crate::storage::{
    EncryptedStore, decode_encrypted_data, decode_nonce, decode_salt, encode_encrypted_data,
    encode_nonce, encode_salt, load_encrypted_store, save_encrypted_store,
};

/// The password manager.
pub struct Manager {
    /// Stored credentials.
    credentials: Credentials,
    /// Path to the password database.
    pwd_db_path: Option<PathBuf>,
    /// Master password (kept only while needed).
    master_password: Option<String>,
}

impl Manager {
    /// Creates a new manager.
    pub fn new() -> Self {
        Self {
            credentials: Credentials::new(),
            pwd_db_path: None,
            master_password: None,
        }
    }

    /// Sets the database path.
    pub fn set_db_path(&mut self, path: PathBuf) {
        self.pwd_db_path = Some(path);
    }

    /// Checks if this is a new user (no existing database).
    pub fn is_new_user(&self) -> bool {
        match &self.pwd_db_path {
            Some(path) => {
                !path.exists() || fs::metadata(path).map(|m| m.len() == 0).unwrap_or(true)
            }
            None => true,
        }
    }

    /// Sets up a new user with the given master password.
    pub fn setup_new_user(&mut self, master_password: String) -> Result<()> {
        if self.pwd_db_path.is_none() {
            return Err(anyhow!("Database path not set"));
        }

        self.master_password = Some(master_password);
        self.credentials = Credentials::new();

        // Save empty credentials to create the file
        self.save_credentials()
    }

    /// Validates the master password by attempting to load credentials.
    pub fn validate_master_password(&mut self, password: String) -> Result<bool> {
        let path = self
            .pwd_db_path
            .as_ref()
            .ok_or_else(|| anyhow!("Database path not set"))?;

        if !path.exists() {
            return Ok(false);
        }

        // Try to load credentials with the provided password
        match self.load_credentials_with_password(password.clone()) {
            Ok(_) => {
                self.master_password = Some(password);
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Loads credentials using the provided password.
    fn load_credentials_with_password(&mut self, password: String) -> Result<()> {
        let path = self
            .pwd_db_path
            .as_ref()
            .ok_or_else(|| anyhow!("Database path not set"))?;

        let store = load_encrypted_store(path)?;

        // Decode salt from base64
        let salt = decode_salt(&store.argon2_salt)?;

        // Derive key from password using Argon2id
        let key = derive_key(&password, &salt)?;

        // Decode nonce and encrypted data from base64
        let nonce_bytes = decode_nonce(&store.encryption_nonce)?;
        let encrypted_data = decode_encrypted_data(&store.encrypted_data)?;

        // Decrypt the data
        let nonce_array: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| anyhow!("Invalid nonce length"))?;
        let decrypted_data = decrypt(&encrypted_data, &key, &nonce_array)?;

        // Deserialize the decrypted data
        let credentials_map: HashMap<String, String> = serde_json::from_slice(&decrypted_data)?;
        self.credentials = Credentials::from_map(credentials_map);

        log::info!("Loaded {} credentials", self.credentials.list().len());
        Ok(())
    }

    /// Saves credentials to disk.
    pub fn save_credentials(&self) -> Result<()> {
        let path = self
            .pwd_db_path
            .as_ref()
            .ok_or_else(|| anyhow!("Database path not set"))?;

        let password = self
            .master_password
            .as_ref()
            .ok_or_else(|| anyhow!("Master password not set"))?;

        // Generate salt for Argon2id
        let salt = generate_salt()?;

        // Derive encryption key from master password using Argon2id
        let key = derive_key(password, &salt)?;

        // Serialize credentials to JSON
        let credentials_map = self.credentials.to_map();
        let credentials_json = serde_json::to_vec(credentials_map)?;

        // Generate nonce for encryption
        let nonce_bytes = generate_nonce()?;

        // Encrypt the credentials
        let encrypted_data = encrypt(&credentials_json, &key, &nonce_bytes)?;

        // Create the encrypted store
        let store = EncryptedStore {
            version: 1,
            argon2_salt: encode_salt(&salt),
            encryption_nonce: encode_nonce(&nonce_bytes),
            encrypted_data: encode_encrypted_data(&encrypted_data),
        };

        // Write to file
        save_encrypted_store(path, &store)?;

        log::info!("Saved {} credentials", self.credentials.list().len());
        Ok(())
    }

    /// Clears the master password from memory.
    pub fn clear_master_password(&mut self) {
        if let Some(ref mut pwd) = self.master_password {
            pwd.clear();
        }
        self.master_password = None;
    }

    /// Returns a reference to credentials.
    #[allow(unused)]
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    /// Returns a mutable reference to credentials.
    #[allow(unused)]
    pub fn credentials_mut(&mut self) -> &mut Credentials {
        &mut self.credentials
    }

    /// Runs the interactive shell.
    pub fn run(&mut self) -> Result<()> {
        // Configure history
        let history_path = get_history_path().unwrap_or_else(|_| PathBuf::from("history"));
        let history_config =
            HistoryConfig::new(history_path).with_max_entries(DEFAULT_HISTORY_SIZE);

        let shell_config = ShellConfig {
            history: history_config,
            show_welcome: true,
        };

        let shell = Shell::with_config(shell_config);

        // We need to clone the necessary data for the save closure
        let pwd_db_path = self.pwd_db_path.clone();
        let master_password = self.master_password.clone();

        // Run shell with save callback
        shell.run_with_save(&mut self.credentials, |credentials| {
            save_credentials_impl(&pwd_db_path, &master_password, credentials)
        })?;

        // Clear password on exit
        self.clear_master_password();

        Ok(())
    }
}

/// Internal function to save credentials (used by closure).
fn save_credentials_impl(
    pwd_db_path: &Option<PathBuf>,
    master_password: &Option<String>,
    credentials: &Credentials,
) -> Result<()> {
    let path = pwd_db_path
        .as_ref()
        .ok_or_else(|| anyhow!("Database path not set"))?;

    let password = master_password
        .as_ref()
        .ok_or_else(|| anyhow!("Master password not set"))?;

    // Generate salt for Argon2id
    let salt = generate_salt()?;

    // Derive encryption key from master password using Argon2id
    let key = derive_key(password, &salt)?;

    // Serialize credentials to JSON
    let credentials_map = credentials.to_map();
    let credentials_json = serde_json::to_vec(credentials_map)?;

    // Generate nonce for encryption
    let nonce_bytes = generate_nonce()?;

    // Encrypt the credentials
    let encrypted_data = encrypt(&credentials_json, &key, &nonce_bytes)?;

    // Create the encrypted store
    let store = EncryptedStore {
        version: 1,
        argon2_salt: encode_salt(&salt),
        encryption_nonce: encode_nonce(&nonce_bytes),
        encrypted_data: encode_encrypted_data(&encrypted_data),
    };

    // Write to file
    save_encrypted_store(path, &store)?;

    log::info!("Saved {} credentials", credentials.list().len());
    Ok(())
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_manager() -> (Manager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut manager = Manager::new();
        manager.set_db_path(db_path);

        (manager, temp_dir)
    }

    #[test]
    fn test_new_manager() {
        let manager = Manager::new();
        assert!(manager.pwd_db_path.is_none());
        assert!(manager.master_password.is_none());
        assert!(manager.credentials.is_empty());
    }

    #[test]
    fn test_is_new_user() {
        let (manager, _temp_dir) = setup_manager();
        assert!(manager.is_new_user());
    }

    #[test]
    fn test_setup_new_user() {
        let (mut manager, _temp_dir) = setup_manager();

        let result = manager.setup_new_user("test_password".to_string());
        assert!(result.is_ok());
        assert!(!manager.is_new_user());
    }

    #[test]
    fn test_validate_password() {
        let (mut manager, _temp_dir) = setup_manager();

        manager
            .setup_new_user("correct_password".to_string())
            .unwrap();

        // Test with correct password
        let result = manager.validate_master_password("correct_password".to_string());
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test with wrong password
        let mut manager2 = Manager::new();
        manager2.set_db_path(manager.pwd_db_path.clone().unwrap());

        let result = manager2.validate_master_password("wrong_password".to_string());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_save_and_load_credentials() {
        let (mut manager, _temp_dir) = setup_manager();

        manager.setup_new_user("test_password".to_string()).unwrap();
        manager
            .credentials_mut()
            .add("key1".to_string(), "value1".to_string())
            .unwrap();
        manager
            .credentials_mut()
            .add("key2".to_string(), "value2".to_string())
            .unwrap();
        manager.save_credentials().unwrap();

        // Create new manager and load
        let mut manager2 = Manager::new();
        manager2.set_db_path(manager.pwd_db_path.clone().unwrap());
        let valid = manager2
            .validate_master_password("test_password".to_string())
            .unwrap();

        assert!(valid);
        assert_eq!(
            manager2.credentials().get("key1"),
            Some(&"value1".to_string())
        );
        assert_eq!(
            manager2.credentials().get("key2"),
            Some(&"value2".to_string())
        );
    }

    #[test]
    fn test_clear_master_password() {
        let (mut manager, _temp_dir) = setup_manager();

        manager.setup_new_user("test_password".to_string()).unwrap();
        assert!(manager.master_password.is_some());

        manager.clear_master_password();
        assert!(manager.master_password.is_none());
    }
}
