use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "passmgr")]
#[command(version = "0.1")]
#[command(about = "Manages your passwords", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
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
        name: String,
    },
    /// Remove a credential by name.
    #[command(alias = "rm")]
    Remove {
        /// Name of the credential to remove
        name: String,
    },
    /// List all stored credentials.
    List,
    /// Quit the password manager.
    #[command(alias = "exit")]
    Quit,
}

