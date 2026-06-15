//! Passmgr - A secure password manager.
//!
//! This is the main entry point for the passmgr binary. With no subcommand it
//! opens the interactive REPL; with a subcommand it performs a single
//! operation and exits (useful for scripting).

mod clipboard;
mod config;
mod credentials;
mod crypto;
mod logging;
mod manager;
mod prompt;
mod shell;
mod storage;
mod trie;

use clap::{Parser, Subcommand};
use config::{get_log_path, get_password_db};
use log::LevelFilter;
use logging::{LogConfig, init_logging};
use manager::Manager;
use std::path::Path;

#[derive(Parser)]
#[command(name = "passmgr", about = "A secure command-line password manager")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Retrieve a credential (copies to clipboard by default).
    Get {
        /// Name of the credential.
        name: String,
        /// Print the secret to the terminal instead of copying it.
        #[arg(long, short)]
        show: bool,
    },
    /// Add a new credential (prompts for the secret with hidden input).
    Add {
        /// Name of the credential.
        name: String,
    },
    /// List all stored credential names.
    List,
    /// Remove a credential.
    Remove {
        /// Name of the credential.
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

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

    let pwd_db = match get_password_db() {
        Ok(path) => path,
        Err(e) => {
            eprintln!(
                "Error: could not determine password database location: {}",
                e
            );
            log::error!("Failed to get database path: {}", e);
            std::process::exit(1);
        }
    };

    let mut manager = Manager::new();
    manager.set_db_path(pwd_db.clone());

    let exit_code = match cli.command {
        None => run_interactive(&mut manager, &pwd_db),
        Some(cmd) => run_command(&mut manager, cmd),
    };

    log::info!("Passmgr exiting");
    std::process::exit(exit_code);
}

/// Runs the interactive REPL (with first-time setup if needed).
fn run_interactive(manager: &mut Manager, pwd_db: &Path) -> i32 {
    println!("Welcome to passmgr!");
    println!("Using password database at: {}", pwd_db.display());

    if manager.is_new_user() {
        if !setup_new_user(manager) {
            return 1;
        }
    } else if !unlock(manager) {
        return 1;
    }

    if let Err(e) = manager.run() {
        eprintln!("Error: {}", e);
        log::error!("Shell error: {}", e);
        return 1;
    }
    0
}

/// Prompts for and validates the master password against an existing database.
/// Returns true on success.
fn unlock(manager: &mut Manager) -> bool {
    println!("Please enter your MASTER password to unlock your credentials.");

    let pwd = match rpassword::prompt_password("Master Password: ") {
        Ok(pwd) => pwd.trim().to_string(),
        Err(_) => {
            eprintln!("Error: failed to read master password");
            log::error!("Failed to read master password");
            return false;
        }
    };

    if pwd.is_empty() {
        eprintln!("Error: master password cannot be empty");
        log::warn!("Empty password attempted");
        return false;
    }

    match manager.validate_master_password(pwd) {
        Ok(true) => {
            println!("Password database unlocked successfully!");
            log::info!("Database unlocked");
            true
        }
        Ok(false) => {
            eprintln!("Error: invalid master password");
            log::warn!("Invalid password attempt");
            false
        }
        Err(e) => {
            eprintln!("Error validating password: {}", e);
            log::error!("Password validation error: {}", e);
            false
        }
    }
}

/// Walks a new user through creating a master password. Returns true on success.
fn setup_new_user(manager: &mut Manager) -> bool {
    println!("No password database found. Let's set up a new one!");
    println!("Please create a MASTER password to encrypt your credentials.");
    println!("IMPORTANT: If you forget this password, your data cannot be recovered!");
    log::info!("Setting up new user");

    let pwd = match rpassword::prompt_password("New Master Password: ") {
        Ok(pwd) => pwd.trim().to_string(),
        Err(_) => {
            eprintln!("Error: failed to read master password");
            log::error!("Failed to read master password");
            return false;
        }
    };

    if pwd.is_empty() {
        eprintln!("Error: master password cannot be empty");
        log::warn!("Empty master password attempted");
        return false;
    }

    let confirm = match rpassword::prompt_password("Confirm Master Password: ") {
        Ok(pwd) => pwd.trim().to_string(),
        Err(_) => {
            eprintln!("Error: failed to read password confirmation");
            log::error!("Failed to read password confirmation");
            return false;
        }
    };

    if pwd != confirm {
        eprintln!("Error: passwords do not match");
        log::warn!("Password confirmation failed");
        return false;
    }

    if let Err(e) = manager.setup_new_user(pwd) {
        eprintln!("Error setting up new user: {}", e);
        log::error!("Failed to setup new user: {}", e);
        return false;
    }

    println!("New password database created successfully!");
    log::info!("New user setup completed");
    true
}

/// Runs a single non-interactive subcommand and returns the process exit code.
fn run_command(manager: &mut Manager, cmd: Commands) -> i32 {
    if manager.is_new_user() {
        eprintln!(
            "Error: no password database found. Run `passmgr` once (with no \
             arguments) to create one before using subcommands."
        );
        return 1;
    }

    if !unlock(manager) {
        return 1;
    }

    match cmd {
        Commands::Get { name, show } => cmd_get(manager, &name, show),
        Commands::Add { name } => cmd_add(manager, &name),
        Commands::List => cmd_list(manager),
        Commands::Remove { name } => cmd_remove(manager, &name),
    }
}

fn cmd_get(manager: &Manager, name: &str, show: bool) -> i32 {
    let secret = match manager.credentials().get(name) {
        Some(s) => s.clone(),
        None => {
            eprintln!("'{}' not found", name);
            return 1;
        }
    };

    if show {
        println!("{}", secret);
        return 0;
    }

    match clipboard::copy_with_autoclear(secret) {
        Ok(handle) => {
            println!(
                "Copied '{}' to clipboard (auto-clears in {}s).",
                name,
                clipboard::AUTOCLEAR_SECS
            );
            // Keep the process alive so the clipboard contents persist and are
            // cleared, then exit.
            let _ = handle.join();
            0
        }
        Err(e) => {
            eprintln!(
                "No clipboard available ({}). Use `passmgr get {} --show` to display the secret.",
                e, name
            );
            1
        }
    }
}

fn cmd_add(manager: &mut Manager, name: &str) -> i32 {
    let secret = match prompt::prompt_secret() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };

    if let Err(e) = manager.credentials_mut().add(name.to_string(), secret) {
        eprintln!("Error: {}", e);
        return 1;
    }

    if let Err(e) = manager.save_credentials() {
        eprintln!("Error saving: {}", e);
        return 1;
    }

    println!("Added '{}'", name);
    0
}

fn cmd_list(manager: &Manager) -> i32 {
    let mut names: Vec<&String> = manager.credentials().list();
    if names.is_empty() {
        println!("No credentials stored.");
        return 0;
    }
    names.sort();
    for name in names {
        println!("{}", name);
    }
    0
}

fn cmd_remove(manager: &mut Manager, name: &str) -> i32 {
    if !manager.credentials_mut().remove(name) {
        eprintln!("'{}' not found", name);
        return 1;
    }

    if let Err(e) = manager.save_credentials() {
        eprintln!("Error saving: {}", e);
        return 1;
    }

    println!("Removed '{}'", name);
    0
}
