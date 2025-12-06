use anyhow::{Result, anyhow};
use crate::credentials::Credentials;
use crate::crypto::{decrypt, encrypt, derive_key, generate_nonce, generate_salt};
use crate::repl::CredentialManager;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::storage::{
    decode_encrypted_data, decode_nonce, decode_salt, encode_encrypted_data, encode_nonce,
    encode_salt, load_encrypted_store, save_encrypted_store, EncryptedStore,
};

pub struct Manager {
    credentials: Credentials,
    pwd_db_path: Option<PathBuf>,
    master_password: Option<String>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            credentials: Credentials::new(),
            pwd_db_path: None,
            master_password: None,
        }
    }

    pub fn set_db_path(&mut self, path: PathBuf) {
        self.pwd_db_path = Some(path);
    }

    pub fn is_new_user(&self) -> bool {
        match &self.pwd_db_path {
            Some(path) => {
                !path.exists() || fs::metadata(path).map(|m| m.len() == 0).unwrap_or(true)
            }
            None => true,
        }
    }

    pub fn setup_new_user(&mut self, master_password: String) -> Result<()> {
        if self.pwd_db_path.is_none() {
            return Err(anyhow!("Database path not set"));
        }

        self.master_password = Some(master_password);
        self.credentials = Credentials::new();

        // Save empty credentials to create the file
        self.do_save_credentials()
    }

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

        Ok(())
    }

    fn do_save_credentials(&self) -> Result<()> {
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

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        crate::repl::run_repl(self)
    }
}

impl CredentialManager for Manager {
    fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    fn credentials_mut(&mut self) -> &mut Credentials {
        &mut self.credentials
    }

    fn save_credentials(&self) -> Result<()> {
        self.do_save_credentials()
    }

    fn clear_master_password(&mut self) {
        if let Some(ref mut pwd) = self.master_password {
            pwd.clear();
        }
    }
}
