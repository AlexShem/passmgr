use anyhow::{Result, anyhow};
use argon2::Argon2;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
use rand::{TryRngCore, rngs::OsRng};

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow!("Failed to derive encryption key using Argon2id: {}", e))?;
    Ok(key)
}

pub fn generate_salt() -> Result<[u8; 16]> {
    let mut salt = [0u8; 16];
    OsRng.try_fill_bytes(&mut salt)?;
    Ok(salt)
}

pub fn generate_nonce() -> Result<[u8; 12]> {
    let mut nonce_bytes = [0u8; 12];
    OsRng.try_fill_bytes(&mut nonce_bytes)?;
    Ok(nonce_bytes)
}

pub fn encrypt(data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    cipher
        .encrypt(nonce, data)
        .map_err(|_| anyhow!("Encryption failed"))
}

pub fn decrypt(encrypted_data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|_| anyhow!("Decryption failed - invalid password"))
}
