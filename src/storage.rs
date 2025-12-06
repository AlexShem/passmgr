use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct EncryptedStore {
    pub version: u8,
    pub argon2_salt: String,      // Base64 encoded
    pub encryption_nonce: String, // Base64 encoded
    pub encrypted_data: String,   // Base64 encoded
}

pub fn load_encrypted_store(path: &Path) -> Result<EncryptedStore> {
    let file_content = fs::read_to_string(path)?;
    if file_content.trim().is_empty() {
        return Err(anyhow!("Password file is empty"));
    }
    let store: EncryptedStore = serde_json::from_str(&file_content)?;
    Ok(store)
}

pub fn save_encrypted_store(path: &Path, store: &EncryptedStore) -> Result<()> {
    let json = serde_json::to_string_pretty(store)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn decode_salt(encoded: &str) -> Result<Vec<u8>> {
    Ok(general_purpose::STANDARD.decode(encoded)?)
}

pub fn decode_nonce(encoded: &str) -> Result<Vec<u8>> {
    Ok(general_purpose::STANDARD.decode(encoded)?)
}

pub fn decode_encrypted_data(encoded: &str) -> Result<Vec<u8>> {
    Ok(general_purpose::STANDARD.decode(encoded)?)
}

pub fn encode_salt(salt: &[u8]) -> String {
    general_purpose::STANDARD.encode(salt)
}

pub fn encode_nonce(nonce: &[u8]) -> String {
    general_purpose::STANDARD.encode(nonce)
}

pub fn encode_encrypted_data(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

