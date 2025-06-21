use clap::Parser;
use std::process::ExitCode;

/// Simple password manager scaffold
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Master password used to unlock the store
    master: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.master.trim().is_empty() {
        eprintln!("Error: master password cannot be empty");
        return ExitCode::from(1);
    }

    println!("Unlocked (stub).");
    ExitCode::SUCCESS
}