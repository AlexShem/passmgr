use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
use argon2::{Argon2};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
use chacha20poly1305::aead::{Aead};
use base64::{Engine as _, engine::general_purpose};
use rand::{rngs::OsRng, TryRngCore};

#[derive(Serialize, Deserialize)]
struct EncryptedStore {
    version: u8,
    argon2_salt: String, // Base64 encoded
    encryption_nonce: String, // Base64 encoded
    encrypted_data: String, // Base64 encoded
}

pub struct Manager {
    credentials: HashMap<String, String>,
    pwd_db_path: Option<PathBuf>,
    master_password: Option<String>,
}

#[derive(Parser)]
#[command(name = "passmgr")]
#[command(version = "0.1")]
#[command(about = "Manages your passwords", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new credential.
    Add {
        /// Name of the credential
        #[arg(short, long)]
        name: String,
        /// Secret value of the credential
        #[arg(short, long)]
        secret: String,
    },
    /// Get a credential by name.
    Get {
        /// Name of the credential to retrieve
        name: String
    },
    /// Remove a credential by name.
    #[command(alias = "rm")]
    Remove {
        /// Name of the credential to remove
        name: String
    },
    /// List all stored credentials.
    List,
    /// Quit the password manager.
    #[command(alias = "exit")]
    Quit,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
            pwd_db_path: None,
            master_password: None,
        }
    }

    pub fn set_db_path(&mut self, path: PathBuf) {
        self.pwd_db_path = Some(path);
    }

    pub fn is_new_user(&self) -> bool {
        match &self.pwd_db_path {
            Some(path) => !path.exists() || fs::metadata(path).map(|m| m.len() == 0).unwrap_or(true),
            None => true,
        }
    }

    pub fn setup_new_user(&mut self, master_password: String) -> Result<()> {
        if self.pwd_db_path.is_none() {
            return Err(anyhow!("Database path not set"));
        }

        self.master_password = Some(master_password);
        self.credentials = HashMap::new();

        // Save empty credentials to create the file
        self.save_credentials()
    }

    pub fn validate_master_password(&mut self, password: String) -> Result<bool> {
        let path = self.pwd_db_path.as_ref()
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
        let path = self.pwd_db_path.as_ref()
            .ok_or_else(|| anyhow!("Database path not set"))?;

        let file_content = fs::read_to_string(path)?;
        if file_content.trim().is_empty() {
            return Err(anyhow!("Password file is empty"));
        }

        let store: EncryptedStore = serde_json::from_str(&file_content)?;

        // Decode salt from base64
        let salt = general_purpose::STANDARD.decode(store.argon2_salt)?;

        // Derive key from password using Argon2id
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];
        argon2.hash_password_into(password.as_bytes(), &salt, &mut key)
            .map_err(|e| anyhow!("Failed to derive encryption key using Argon2id: {}", e))?;

        // Decode nonce and encrypted data from base64
        let nonce_bytes = general_purpose::STANDARD.decode(store.encryption_nonce)?;
        let encrypted_data = general_purpose::STANDARD.decode(store.encrypted_data)?;

        // Decrypt the data
        let cipher = ChaCha20Poly1305::new(&key.into());
        let nonce = Nonce::from_slice(&nonce_bytes);

        let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
            .map_err(|_| anyhow!("Decryption failed - invalid password"))?;

        // Deserialize the decrypted data
        self.credentials = serde_json::from_slice(&decrypted_data)?;

        Ok(())
    }

    fn save_credentials(&self) -> Result<()> {
        let path = self.pwd_db_path.as_ref()
            .ok_or_else(|| anyhow!("Database path not set"))?;

        let password = self.master_password.as_ref()
            .ok_or_else(|| anyhow!("Master password not set"))?;

        // Generate salt for Argon2id
        let mut salt = [0u8; 16];
        // OsRng.fill_bytes(&mut salt);
        OsRng.try_fill_bytes(&mut salt)?;

        // Derive encryption key from master password using Argon2id
        let argon2 = Argon2::default();
        let mut key = [0u8; 32]; // 256-bit key
        argon2.hash_password_into(password.as_bytes(), &salt, &mut key)
            .map_err(|e| anyhow!("Failed to derive encryption key using Argon2id: {}", e))?;

        // Serialize credentials to JSON
        let credentials_json = serde_json::to_vec(&self.credentials)?;

        // Generate nonce for encryption
        let mut nonce_bytes = [0u8; 12];
        OsRng.try_fill_bytes(&mut nonce_bytes)?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the credentials
        let cipher = ChaCha20Poly1305::new(&key.into());
        let encrypted_data = cipher.encrypt(nonce, credentials_json.as_ref())
            .map_err(|_| anyhow!("Encryption failed"))?;

        // Create the encrypted store
        let store = EncryptedStore {
            version: 1,
            argon2_salt: general_purpose::STANDARD.encode(salt),
            encryption_nonce: general_purpose::STANDARD.encode(nonce),
            encrypted_data: general_purpose::STANDARD.encode(encrypted_data),
        };

        // Write to file
        let json = serde_json::to_string_pretty(&store)?;
        fs::write(path, json)?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        println!("Unlocked. Type 'help' for available commands.");

        let mut stdout = io::stdout();

        loop {
            print!("passmgr> ");
            stdout.flush()?;

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let args: Vec<String> = match shell_words::split(&input) {
                Ok(args) => {
                    args
                        .iter()
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>()
                }
                Err(_) => continue
            };

            if args.is_empty() {
                continue;
            }

            let result = Cli::try_parse_from(std::iter::once("passmgr".to_string()).chain(args));
            match result {
                Ok(cli) => {
                    match cli.command {
                        None => {
                            continue;
                        }
                        Some(Commands::Add { name, secret }) => {
                            if self.credentials.contains_key(&name) {
                                println!("Error: '{}' already exists.", name);
                            } else {
                                self.credentials.insert(name.clone(), secret);
                                if let Err(e) = self.save_credentials() {
                                    eprintln!("Failed to save credentials: {}", e);
                                } else {
                                    println!("Added {}", name);
                                }
                            }
                        }
                        Some(Commands::Get { name }) => {
                            match self.credentials.get(&name) {
                                Some(secret) => println!("{}", secret),
                                None => {
                                    eprintln!("Error: {} not found", name);
                                }
                            }
                        }
                        Some(Commands::Remove { name }) => {
                            if self.credentials.remove(&name).is_some() {
                                if let Err(e) = self.save_credentials() {
                                    eprintln!("Failed to save credentials: {}", e);
                                } else {
                                    println!("Removed {}", name);
                                }
                            } else {
                                eprintln!("Error: {} not found", name);
                            }
                        }
                        Some(Commands::List) => {
                            if self.credentials.is_empty() {
                                println!("No credentials stored.");
                            } else {
                                for (name, _) in &self.credentials {
                                    println!("{}", name);
                                }
                            }
                        }
                        Some(Commands::Quit) => {
                            println!("Exiting...");
                            // Securely clear the master password
                            if let Some(ref mut pwd) = self.master_password {
                                pwd.clear();
                            }
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }

        Ok(())
    }
}