use anyhow::{Result, anyhow};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
use rand::{TryRngCore, rngs::OsRng};

/// Argon2id memory cost in KiB. Pinned to match `Argon2::default()` as of
/// argon2 0.5.x so that databases created with the default stay decryptable
/// even if the crate's defaults change in a future version.
const ARGON2_M_COST: u32 = 19456;
/// Argon2id time cost (number of iterations). Pinned (see `ARGON2_M_COST`).
const ARGON2_T_COST: u32 = 2;
/// Argon2id parallelism (lanes). Pinned (see `ARGON2_M_COST`).
const ARGON2_P_COST: u32 = 1;
/// Derived key length in bytes (256-bit ChaCha20-Poly1305 key).
const ARGON2_KEY_LEN: usize = 32;

/// Builds the Argon2id hasher with our pinned parameters.
fn argon2_hasher() -> Result<Argon2<'static>> {
    let params = Params::new(
        ARGON2_M_COST,
        ARGON2_T_COST,
        ARGON2_P_COST,
        Some(ARGON2_KEY_LEN),
    )
    .map_err(|e| anyhow!("Invalid Argon2 parameters: {}", e))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let argon2 = argon2_hasher()?;
    let mut key = [0u8; ARGON2_KEY_LEN];
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Locks the Argon2id parameters: a fixed password + salt must always
    /// derive this exact key. If a dependency bump changes the defaults, this
    /// test fails instead of silently making existing databases undecryptable.
    #[test]
    fn test_derive_key_is_stable() {
        let salt = [0u8; 16];
        let key = derive_key("correct horse battery staple", &salt).unwrap();
        let hex = key.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        assert_eq!(
            hex,
            "bcaf6fd0e5aaa31b272240c38067653313e9f7802fc226ccf8416cf7bcf9e644"
        );
    }

    #[test]
    fn test_derive_key_deterministic() {
        let salt = [7u8; 16];
        let a = derive_key("pw", &salt).unwrap();
        let b = derive_key("pw", &salt).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_derive_key_salt_sensitive() {
        let a = derive_key("pw", &[1u8; 16]).unwrap();
        let b = derive_key("pw", &[2u8; 16]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let nonce = [9u8; 12];
        let plaintext = b"super secret value";
        let ct = encrypt(plaintext, &key, &nonce).unwrap();
        let pt = decrypt(&ct, &key, &nonce).unwrap();
        assert_eq!(pt, plaintext);
    }
}
