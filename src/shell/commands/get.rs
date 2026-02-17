//! Get command implementation.

use crate::shell::command::{Command, CommandResult, ShellContext};

/// Command to retrieve a credential.
pub struct GetCommand;

impl Command for GetCommand {
    fn name(&self) -> &str {
        "get"
    }

    fn aliases(&self) -> &[&str] {
        &["g", "show"]
    }

    fn description(&self) -> &str {
        "Get a credential by name"
    }

    fn usage(&self) -> &str {
        "get <name>"
    }

    fn help(&self) -> &str {
        "Retrieve and display a stored credential.\n\n\
         Arguments:\n  \
           <name> - The name of the credential to retrieve\n\n\
         Examples:\n  \
           get github\n  \
           get \"my email\""
    }

    fn execute(&self, args: &[&str], ctx: &mut ShellContext) -> CommandResult {
        if args.is_empty() {
            return CommandResult::error(format!(
                "Usage: {}\nMissing credential name",
                self.usage()
            ));
        }

        let name = args[0];
        log::debug!("Getting credential: {}", name);

        match ctx.credentials.get(name) {
            Some(secret) => {
                log::info!("Retrieved credential: {}", name);
                CommandResult::success(secret.clone())
            }
            None => {
                log::debug!("Credential not found: {}", name);
                CommandResult::error(format!("'{}' not found", name))
            }
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
    fn test_get_command_success() {
        let mut credentials = Credentials::new();
        credentials
            .add("test_key".to_string(), "test_secret".to_string())
            .unwrap();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = GetCommand;
        let result = cmd.execute(&["test_key"], &mut ctx);

        match result {
            CommandResult::Success(Some(msg)) => assert_eq!(msg, "test_secret"),
            _ => panic!("Expected success with secret"),
        }
    }

    #[test]
    fn test_get_command_not_found() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = GetCommand;
        let result = cmd.execute(&["unknown"], &mut ctx);

        assert!(matches!(result, CommandResult::Error(_)));
    }

    #[test]
    fn test_get_command_missing_args() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = GetCommand;
        let result = cmd.execute(&[], &mut ctx);

        assert!(matches!(result, CommandResult::Error(_)));
    }

    #[test]
    fn test_get_command_completions() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        trie.insert("github");
        trie.insert("gitlab");
        trie.insert("email");
        let ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = GetCommand;
        let completions = cmd.completions(0, "gi", &ctx);

        assert!(completions.contains(&"github".to_string()));
        assert!(completions.contains(&"gitlab".to_string()));
        assert!(!completions.contains(&"email".to_string()));
    }
}
