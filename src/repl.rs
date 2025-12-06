use anyhow::Result;
use clap::Parser;
use crate::cli::{Cli, Commands};
use crate::credentials::Credentials;
use std::io;
use std::io::Write;

pub trait CredentialManager {
    fn credentials(&self) -> &Credentials;
    fn credentials_mut(&mut self) -> &mut Credentials;
    fn save_credentials(&self) -> Result<()>;
    fn clear_master_password(&mut self);
}

pub fn run_repl<M: CredentialManager>(manager: &mut M) -> Result<()> {
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
            Ok(args) => args
                .iter()
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>(),
            Err(_) => continue,
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
                        match manager.credentials_mut().add(name.clone(), secret) {
                            Ok(_) => {
                                if let Err(e) = manager.save_credentials() {
                                    eprintln!("Failed to save credentials: {}", e);
                                } else {
                                    println!("Added {}", name);
                                }
                            }
                            Err(e) => {
                                println!("Error: {}", e);
                            }
                        }
                    }
                    Some(Commands::Get { name }) => match manager.credentials().get(&name) {
                        Some(secret) => println!("{}", secret),
                        None => {
                            eprintln!("Error: {} not found", name);
                        }
                    },
                    Some(Commands::Remove { name }) => {
                        if manager.credentials_mut().remove(&name) {
                            if let Err(e) = manager.save_credentials() {
                                eprintln!("Failed to save credentials: {}", e);
                            } else {
                                println!("Removed {}", name);
                            }
                        } else {
                            eprintln!("Error: {} not found", name);
                        }
                    }
                    Some(Commands::List) => {
                        if manager.credentials().is_empty() {
                            println!("No credentials stored.");
                        } else {
                            for name in manager.credentials().list() {
                                println!("{}", name);
                            }
                        }
                    }
                    Some(Commands::Quit) => {
                        println!("Exiting...");
                        manager.clear_master_password();
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

