//! Clear command implementation.

use std::io::Write;

use crate::shell::command::{Command, CommandResult, ShellContext};

/// Command to clear the terminal screen (e.g. to hide a revealed secret).
pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }

    fn aliases(&self) -> &[&str] {
        &["cls"]
    }

    fn description(&self) -> &str {
        "Clear the terminal screen"
    }

    fn usage(&self) -> &str {
        "clear"
    }

    fn help(&self) -> &str {
        "Clear the terminal screen and scrollback, useful for hiding a secret\n\
         that was printed with 'get --show'."
    }

    fn execute(&self, _args: &[&str], _ctx: &mut ShellContext) -> CommandResult {
        // ANSI: clear screen (2J), clear scrollback (3J), move cursor home (H).
        print!("\x1b[2J\x1b[3J\x1b[H");
        let _ = std::io::stdout().flush();
        CommandResult::ok()
    }

    fn max_args(&self) -> Option<usize> {
        Some(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::Credentials;
    use crate::trie::Trie;

    #[test]
    fn test_clear_returns_ok_without_message() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = ClearCommand;
        let result = cmd.execute(&[], &mut ctx);
        assert!(matches!(result, CommandResult::Success(None)));
    }
}
