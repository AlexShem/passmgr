//! Add command implementation.

use crate::shell::command::{Command, CommandResult, ShellContext};

/// Command to add a new credential.
pub struct AddCommand;

impl Command for AddCommand {
    fn name(&self) -> &str {
        "add"
    }

    fn aliases(&self) -> &[&str] {
        &["a", "new", "set"]
    }

    fn description(&self) -> &str {
        "Add a new credential"
    }

    fn usage(&self) -> &str {
        "add <name> <secret>"
    }

    fn help(&self) -> &str {
        "Add a new credential to the store.\n\n\
         Arguments:\n  \
           <name>   - Unique identifier for the credential\n  \
           <secret> - The secret value to store\n\n\
         Examples:\n  \
           add github mypassword123\n  \
           add \"my email\" \"secret with spaces\""
    }

    fn execute(&self, args: &[&str], ctx: &mut ShellContext) -> CommandResult {
        if args.len() < 2 {
            return CommandResult::error(format!(
                "Usage: {}\nMissing required arguments",
                self.usage()
            ));
        }

        let name = args[0].to_string();
        let secret = args[1..].join(" ");

        log::debug!("Adding credential: {}", name);

        match ctx.credentials.add(name.clone(), secret) {
            Ok(_) => {
                // Update the key trie for autocomplete
                ctx.key_trie.insert(&name);
                ctx.mark_modified();
                log::info!("Added credential: {}", name);
                CommandResult::success(format!("Added '{}'", name))
            }
            Err(e) => {
                log::warn!("Failed to add credential '{}': {}", name, e);
                CommandResult::error(e)
            }
        }
    }

    fn completions(&self, _arg_index: usize, _partial: &str, _ctx: &ShellContext) -> Vec<String> {
        // No completions for add command (name should be new)
        vec![]
    }

    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> Option<usize> {
        None // Allow spaces in secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::Credentials;
    use crate::trie::Trie;

    #[test]
    fn test_add_command_success() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = AddCommand;
        let result = cmd.execute(&["test_key", "test_secret"], &mut ctx);

        assert!(matches!(result, CommandResult::Success(_)));
        assert!(ctx.modified);
        assert_eq!(
            credentials.get("test_key"),
            Some(&"test_secret".to_string())
        );
    }

    #[test]
    fn test_add_command_missing_args() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = AddCommand;
        let result = cmd.execute(&["only_name"], &mut ctx);

        assert!(matches!(result, CommandResult::Error(_)));
        assert!(!ctx.modified);
    }

    #[test]
    fn test_add_command_duplicate() {
        let mut credentials = Credentials::new();
        credentials
            .add("existing".to_string(), "value".to_string())
            .unwrap();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = AddCommand;
        let result = cmd.execute(&["existing", "new_value"], &mut ctx);

        assert!(matches!(result, CommandResult::Error(_)));
    }

    #[test]
    fn test_add_command_secret_with_spaces() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = AddCommand;
        let result = cmd.execute(&["key", "secret", "with", "spaces"], &mut ctx);

        assert!(matches!(result, CommandResult::Success(_)));
        assert_eq!(
            credentials.get("key"),
            Some(&"secret with spaces".to_string())
        );
    }
}
