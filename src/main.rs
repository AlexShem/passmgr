use std::path::PathBuf;

mod manager;

fn main() {
    println!("Welcome to passmgr!");

    let pwd_db = match get_password_db() {
        Ok(path) => {
            println!("Using password database at: {}", path.display());
            path
        }
        Err(e) => {
            eprintln!("Error: could not determine password database location: {}", e);
            return;
        }
    };

    let mut manager = manager::Manager::new();
    manager.set_db_path(pwd_db);

    if manager.is_new_user() {
        println!("No password database found. Let's set up a new one!");
        println!("Please create a MASTER password to encrypt your credentials.");
        println!("IMPORTANT: If you forget this password, your data cannot be recovered!");

        match rpassword::prompt_password("New Master Password: ") {
            Ok(pwd) => {
                let pwd = pwd.trim().to_string();
                if pwd.is_empty() {
                    eprintln!("Error: master password cannot be empty");
                    return;
                }

                match rpassword::prompt_password("Confirm Master Password: ") {
                    Ok(confirm_pwd) => {
                        let confirm_pwd = confirm_pwd.trim().to_string();
                        if pwd != confirm_pwd {
                            eprintln!("Error: passwords do not match");
                            return;
                        }

                        if let Err(e) = manager.setup_new_user(pwd) {
                            eprintln!("Error setting up new user: {}", e);
                            return;
                        }

                        println!("New password database created successfully!");
                    }
                    Err(_) => {
                        eprintln!("Error: failed to read password confirmation");
                        return;
                    }
                }
            }
            Err(_) => {
                eprintln!("Error: failed to read master password");
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
                    return;
                }

                match manager.validate_master_password(pwd) {
                    Ok(true) => {
                        println!("Password database unlocked successfully!");
                    }
                    Ok(false) => {
                        eprintln!("Error: invalid master password");
                        return;
                    }
                    Err(e) => {
                        eprintln!("Error validating password: {}", e);
                        return;
                    }
                }
            }
            Err(_) => {
                eprintln!("Error: failed to read master password");
                return;
            }
        }
    }

    if let Err(e) = manager.run() {
        eprintln!("Error: {}", e);
    }
}

fn get_password_db() -> anyhow::Result<PathBuf> {
    if let Some(home_path) = dirs_next::home_dir() {
        let db_path = home_path.join(".passmgr").join("passwords.db");
        if !db_path.exists() {
            std::fs::create_dir_all(db_path.parent().unwrap())?;
            std::fs::File::create(&db_path)?;
        }
        Ok(db_path)
    } else {
        Err(anyhow::anyhow!("Could not determine home directory"))
    }
}