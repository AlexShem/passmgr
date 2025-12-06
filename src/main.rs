//! Passmgr - A secure password manager.
//!
//! This is the main entry point for the passmgr binary.

mod config;
mod credentials;
mod crypto;
mod logging;
mod manager;
mod shell;
mod storage;
mod trie;

use config::{get_log_path, get_password_db};
use log::LevelFilter;
use logging::{LogConfig, init_logging};
use manager::Manager;

fn main() {
    // Initialize logging
    if let Ok(log_path) = get_log_path() {
        let log_config = LogConfig::new(log_path)
            .with_level(LevelFilter::Info)
            .with_max_size(100);
        if let Err(e) = init_logging(&log_config) {
            eprintln!("Warning: Failed to initialize logging: {}", e);
        }
    }

    log::info!("Passmgr starting");
    println!("Welcome to passmgr!");

    let pwd_db = match get_password_db() {
        Ok(path) => {
            println!("Using password database at: {}", path.display());
            log::debug!("Database path: {}", path.display());
            path
        }
        Err(e) => {
            eprintln!(
                "Error: could not determine password database location: {}",
                e
            );
            log::error!("Failed to get database path: {}", e);
            return;
        }
    };

    let mut manager = Manager::new();
    manager.set_db_path(pwd_db);

    if manager.is_new_user() {
        println!("No password database found. Let's set up a new one!");
        println!("Please create a MASTER password to encrypt your credentials.");
        println!("IMPORTANT: If you forget this password, your data cannot be recovered!");

        log::info!("Setting up new user");

        match rpassword::prompt_password("New Master Password: ") {
            Ok(pwd) => {
                let pwd = pwd.trim().to_string();
                if pwd.is_empty() {
                    eprintln!("Error: master password cannot be empty");
                    log::warn!("Empty master password attempted");
                    return;
                }

                match rpassword::prompt_password("Confirm Master Password: ") {
                    Ok(confirm_pwd) => {
                        let confirm_pwd = confirm_pwd.trim().to_string();
                        if pwd != confirm_pwd {
                            eprintln!("Error: passwords do not match");
                            log::warn!("Password confirmation failed");
                            return;
                        }

                        if let Err(e) = manager.setup_new_user(pwd) {
                            eprintln!("Error setting up new user: {}", e);
                            log::error!("Failed to setup new user: {}", e);
                            return;
                        }

                        println!("New password database created successfully!");
                        log::info!("New user setup completed");
                    }
                    Err(_) => {
                        eprintln!("Error: failed to read password confirmation");
                        log::error!("Failed to read password confirmation");
                        return;
                    }
                }
            }
            Err(_) => {
                eprintln!("Error: failed to read master password");
                log::error!("Failed to read master password");
                return;
            }
        }
    } else {
        println!("Please enter your MASTER password to unlock your credentials.");

        match rpassword::prompt_password("Master Password: ") {
            Ok(pwd) => {
                let pwd = pwd.trim().to_string();
                if pwd.is_empty() {
                    eprintln!("Error: master password cannot be empty");
                    log::warn!("Empty password attempted");
                    return;
                }

                match manager.validate_master_password(pwd) {
                    Ok(true) => {
                        println!("Password database unlocked successfully!");
                        log::info!("Database unlocked");
                    }
                    Ok(false) => {
                        eprintln!("Error: invalid master password");
                        log::warn!("Invalid password attempt");
                        return;
                    }
                    Err(e) => {
                        eprintln!("Error validating password: {}", e);
                        log::error!("Password validation error: {}", e);
                        return;
                    }
                }
            }
            Err(_) => {
                eprintln!("Error: failed to read master password");
                log::error!("Failed to read master password");
                return;
            }
        }
    }

    if let Err(e) = manager.run() {
        eprintln!("Error: {}", e);
        log::error!("Shell error: {}", e);
    }

    log::info!("Passmgr exiting");
}
