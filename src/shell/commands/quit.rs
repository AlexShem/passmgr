//! Quit command implementation.

use crate::shell::command::{Command, CommandResult, ShellContext};

/// Command to exit the shell.
pub struct QuitCommand;

impl Command for QuitCommand {
    fn name(&self) -> &str {
        "quit"
    }

    fn aliases(&self) -> &[&str] {
        &["exit", "q"]
    }

    fn description(&self) -> &str {
        "Exit the password manager"
    }

    fn usage(&self) -> &str {
        "quit"
    }

    fn help(&self) -> &str {
        "Exit the password manager and save any pending changes.\n\n\
         Examples:\n  \
           quit\n  \
           exit\n  \
           q"
    }

    fn execute(&self, _args: &[&str], _ctx: &mut ShellContext) -> CommandResult {
        log::info!("User requested exit");
        CommandResult::Exit
    }

    fn min_args(&self) -> usize {
        0
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
    fn test_quit_command() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = QuitCommand;
        let result = cmd.execute(&[], &mut ctx);

        assert!(matches!(result, CommandResult::Exit));
    }
}
