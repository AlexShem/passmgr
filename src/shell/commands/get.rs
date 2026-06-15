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
        "get <name> [--show]"
    }

    fn help(&self) -> &str {
        "Retrieve a stored credential.\n\n\
         By default the secret is copied to the clipboard (and cleared after a\n\
         few seconds) so it never appears on screen. Pass --show to print it.\n\n\
         Arguments:\n  \
           <name>  - The name of the credential to retrieve\n  \
           --show  - Print the secret to the terminal instead of copying it\n\n\
         Examples:\n  \
           get github            (copies to clipboard)\n  \
           get github --show     (prints the secret)"
    }

    fn record_history(&self) -> bool {
        // Keep credential lookups (and their sensitive names) out of history.
        false
    }

    fn execute(&self, args: &[&str], ctx: &mut ShellContext) -> CommandResult {
        // Separate flags from positional arguments.
        let show = args.iter().any(|a| *a == "--show" || *a == "-s");
        let name = match args.iter().find(|a| !a.starts_with('-')) {
            Some(n) => *n,
            None => {
                return CommandResult::error(format!(
                    "Usage: {}\nMissing credential name",
                    self.usage()
                ));
            }
        };

        log::debug!("Getting credential: {} (show={})", name, show);

        let secret = match ctx.credentials.get(name) {
            Some(secret) => secret.clone(),
            None => {
                log::debug!("Credential not found: {}", name);
                return CommandResult::error(format!("'{}' not found", name));
            }
        };

        log::info!("Retrieved credential: {}", name);

        if show {
            return CommandResult::success(secret);
        }

        // Default: copy to the clipboard, never print.
        match crate::clipboard::copy_with_autoclear(secret) {
            Ok(_handle) => CommandResult::success(format!(
                "Copied '{}' to clipboard (auto-clears in {}s). Use --show to print.",
                name,
                crate::clipboard::AUTOCLEAR_SECS
            )),
            Err(e) => {
                log::warn!("Clipboard unavailable: {}", e);
                CommandResult::error(format!(
                    "No clipboard available ({}). Use 'get {} --show' to display the secret.",
                    e, name
                ))
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
    fn test_get_command_show_prints_secret() {
        let mut credentials = Credentials::new();
        credentials
            .add("test_key".to_string(), "test_secret".to_string())
            .unwrap();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = GetCommand;
        let result = cmd.execute(&["test_key", "--show"], &mut ctx);

        match result {
            CommandResult::Success(Some(msg)) => assert_eq!(msg, "test_secret"),
            _ => panic!("Expected success with secret"),
        }
    }

    #[test]
    fn test_get_command_default_never_prints_secret() {
        let mut credentials = Credentials::new();
        credentials
            .add("test_key".to_string(), "test_secret".to_string())
            .unwrap();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = GetCommand;
        // Without --show the secret must never appear in the output, whether a
        // clipboard is available (Success status) or not (Error fallback).
        let result = cmd.execute(&["test_key"], &mut ctx);
        match result {
            CommandResult::Success(Some(msg)) => assert!(!msg.contains("test_secret")),
            CommandResult::Error(msg) => assert!(!msg.contains("test_secret")),
            other => panic!("Unexpected result: {:?}", other),
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
