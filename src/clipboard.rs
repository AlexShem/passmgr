//! Clipboard support shared by the REPL `get` command and the non-interactive
//! CLI. Secrets are copied to the system clipboard and automatically cleared
//! after a short window so they don't linger.
//!
//! Platform note: on X11 the clipboard contents are served by the owning
//! process, so the [`arboard::Clipboard`] instance must stay alive for the
//! auto-clear window. We keep it alive inside the spawned thread; the REPL
//! detaches that thread while the CLI joins it (so a one-shot `passmgr get`
//! lives long enough for the paste to register).

use anyhow::{Result, anyhow};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use arboard::Clipboard;

/// How long a copied secret stays on the clipboard before being cleared.
pub const AUTOCLEAR_SECS: u64 = 15;

/// Copies `secret` to the clipboard and schedules it to be cleared after
/// [`AUTOCLEAR_SECS`].
///
/// Returns a [`JoinHandle`] for the thread that owns the clipboard and performs
/// the clear. Detach it (drop) to clear in the background, or join it to block
/// until the window elapses (needed for short-lived CLI invocations).
///
/// Fails if no clipboard is available (e.g. a headless or SSH session with no
/// display); callers must fall back without revealing the secret.
pub fn copy_with_autoclear(secret: String) -> Result<JoinHandle<()>> {
    let mut clipboard =
        Clipboard::new().map_err(|e| anyhow!("Clipboard is not available: {}", e))?;
    clipboard
        .set_text(secret.clone())
        .map_err(|e| anyhow!("Failed to copy to clipboard: {}", e))?;

    let handle = thread::spawn(move || {
        // Keep `clipboard` alive so the contents remain pasteable on X11.
        thread::sleep(Duration::from_secs(AUTOCLEAR_SECS));
        // Only clear if we still own the same secret, so we don't wipe
        // something the user copied in the meantime.
        if let Ok(current) = clipboard.get_text()
            && current == secret
        {
            let _ = clipboard.set_text(String::new());
        }
    });

    Ok(handle)
}
