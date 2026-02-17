//! Remove command implementation.

use crate::shell::command::{Command, CommandResult, ShellContext};

/// Command to remove a credential.
pub struct RemoveCommand;

impl Command for RemoveCommand {
    fn name(&self) -> &str {
        "remove"
    }

    fn aliases(&self) -> &[&str] {
        &["rm", "delete", "del"]
    }

    fn description(&self) -> &str {
        "Remove a credential by name"
    }

    fn usage(&self) -> &str {
        "remove <name>"
    }

    fn help(&self) -> &str {
        "Remove a credential from the store.\n\n\
         Arguments:\n  \
           <name> - The name of the credential to remove\n\n\
         Examples:\n  \
           remove github\n  \
           rm \"old email\""
    }

    fn execute(&self, args: &[&str], ctx: &mut ShellContext) -> CommandResult {
        if args.is_empty() {
            return CommandResult::error(format!(
                "Usage: {}\nMissing credential name",
                self.usage()
            ));
        }

        let name = args[0];
        log::debug!("Removing credential: {}", name);

        if ctx.credentials.remove(name) {
            // Update the key trie
            ctx.key_trie.remove(name);
            ctx.mark_modified();
            log::info!("Removed credential: {}", name);
            CommandResult::success(format!("Removed '{}'", name))
        } else {
            log::debug!("Credential not found for removal: {}", name);
            CommandResult::error(format!("'{}' not found", name))
        }
    }

    fn completions(&self, arg_index: usize, partial: &str, ctx: &ShellContext) -> Vec<String> {
        if arg_index == 0 {
            // Complete credential names
            ctx.key_trie.completions(partial)
        } else {
            vec![]
        }
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::Credentials;
    use crate::trie::Trie;

    #[test]
    fn test_remove_command_success() {
        let mut credentials = Credentials::new();
        credentials
            .add("test_key".to_string(), "test_secret".to_string())
            .unwrap();
        let mut trie = Trie::new();
        trie.insert("test_key");
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = RemoveCommand;
        let result = cmd.execute(&["test_key"], &mut ctx);

        assert!(matches!(result, CommandResult::Success(_)));
        assert!(ctx.modified);
        assert!(credentials.get("test_key").is_none());
        assert!(!trie.contains("test_key"));
    }

    #[test]
    fn test_remove_command_not_found() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = RemoveCommand;
        let result = cmd.execute(&["unknown"], &mut ctx);

        assert!(matches!(result, CommandResult::Error(_)));
        assert!(!ctx.modified);
    }

    #[test]
    fn test_remove_command_missing_args() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = RemoveCommand;
        let result = cmd.execute(&[], &mut ctx);

        assert!(matches!(result, CommandResult::Error(_)));
    }
}
