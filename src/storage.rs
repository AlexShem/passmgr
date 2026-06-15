use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// The on-disk store format version this build writes and can read.
///
/// Bump this only alongside a documented migration path; `load_encrypted_store`
/// rejects any other version so an older binary never silently mishandles a
/// newer file.
pub const CURRENT_STORE_VERSION: u8 = 1;

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
    if store.version != CURRENT_STORE_VERSION {
        return Err(anyhow!(
            "Unsupported password database version {} (this build supports version {})",
            store.version,
            CURRENT_STORE_VERSION
        ));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_store(dir: &TempDir, version: u8) -> std::path::PathBuf {
        let path = dir.path().join("store.db");
        let store = EncryptedStore {
            version,
            argon2_salt: encode_salt(&[0u8; 16]),
            encryption_nonce: encode_nonce(&[0u8; 12]),
            encrypted_data: encode_encrypted_data(b"data"),
        };
        save_encrypted_store(&path, &store).unwrap();
        path
    }

    #[test]
    fn test_load_current_version_succeeds() {
        let dir = TempDir::new().unwrap();
        let path = write_store(&dir, CURRENT_STORE_VERSION);
        assert!(load_encrypted_store(&path).is_ok());
    }

    #[test]
    fn test_load_unknown_version_rejected() {
        let dir = TempDir::new().unwrap();
        let path = write_store(&dir, CURRENT_STORE_VERSION + 1);
        let err = load_encrypted_store(&path)
            .map(|_| ())
            .expect_err("unknown version should be rejected");
        assert!(err.to_string().contains("Unsupported"));
    }
}
