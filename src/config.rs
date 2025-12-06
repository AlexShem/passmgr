use anyhow::Result;
use std::path::PathBuf;

pub fn get_password_db() -> Result<PathBuf> {
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

