use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::process::ExitCode;
use clap::{Parser, Subcommand};

pub struct Manager {
    credentials: HashMap<String, String>,
    #[allow(dead_code)]
    master_password: String,
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
    /// List all stored credentials.
    List,
    /// Quit the password manager.
    #[command(alias = "exit")]
    Quit,
}

impl Manager {
    pub fn new(master_password: String) -> Self {
        Self {
            credentials: HashMap::new(),
            master_password,
        }
    }

    pub fn run(&mut self) -> ExitCode {
        println!("Unlocked (stub).");

        let mut stdout = io::stdout();

        loop {
            print!("passmgr> ");
            stdout.flush().unwrap();

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
                                println!("Added {}", name);
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
                            return ExitCode::SUCCESS;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }

        ExitCode::SUCCESS
    }
}