//! Hidden secret-entry prompt shared by the REPL `add` command and the
//! non-interactive CLI, so neither path echoes a secret to the terminal.

use anyhow::{Result, anyhow};

/// Prompts for a secret with hidden input and a confirmation entry.
///
/// The value is returned verbatim (not trimmed) so secrets are stored exactly
/// as typed. Errors if the two entries differ or the secret is empty.
pub fn prompt_secret() -> Result<String> {
    let secret = rpassword::prompt_password("Secret: ")?;
    let confirm = rpassword::prompt_password("Confirm secret: ")?;
    if secret != confirm {
        return Err(anyhow!("Secrets do not match"));
    }
    if secret.is_empty() {
        return Err(anyhow!("Secret cannot be empty"));
    }
    Ok(secret)
}
