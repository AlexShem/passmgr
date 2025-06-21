mod manager;

use std::process::ExitCode;

fn main() -> ExitCode {
    println!("Welcome to passmgr!");
    println!("Please enter your MASTER password to unlock your credentials.");

    let master_pwd = rpassword::prompt_password("Master Password: ")
        .unwrap_or_else(|_| {
            eprintln!("Error reading master password");
            std::process::exit(1);
        });

    if master_pwd.is_empty() {
        eprintln!("Error: master password cannot be empty");
        return ExitCode::from(1);
    }

    let mut manager = manager::Manager::new(master_pwd.to_string());
    manager.run();
    ExitCode::SUCCESS
}