//! List command implementation.

use crate::shell::command::{Command, CommandResult, ShellContext};

/// Command to list all credentials.
pub struct ListCommand;

impl Command for ListCommand {
    fn name(&self) -> &str {
        "list"
    }

    fn aliases(&self) -> &[&str] {
        &["ls", "l"]
    }

    fn description(&self) -> &str {
        "List all stored credentials"
    }

    fn usage(&self) -> &str {
        "list"
    }

    fn help(&self) -> &str {
        "Display a list of all stored credential names.\n\n\
         The secrets are not shown, only the names.\n\n\
         Examples:\n  \
           list\n  \
           ls"
    }

    fn execute(&self, _args: &[&str], ctx: &mut ShellContext) -> CommandResult {
        log::debug!("Listing credentials");

        if ctx.credentials.is_empty() {
            return CommandResult::success("No credentials stored.");
        }

        let mut names: Vec<&String> = ctx.credentials.list();
        names.sort();

        let output = names
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        log::info!("Listed {} credentials", names.len());
        CommandResult::success(output)
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
    fn test_list_command_empty() {
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = ListCommand;
        let result = cmd.execute(&[], &mut ctx);

        match result {
            CommandResult::Success(Some(msg)) => {
                assert!(msg.contains("No credentials"));
            }
            _ => panic!("Expected success message"),
        }
    }

    #[test]
    fn test_list_command_with_entries() {
        let mut credentials = Credentials::new();
        credentials
            .add("github".to_string(), "secret1".to_string())
            .unwrap();
        credentials
            .add("email".to_string(), "secret2".to_string())
            .unwrap();
        credentials
            .add("aws".to_string(), "secret3".to_string())
            .unwrap();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie);

        let cmd = ListCommand;
        let result = cmd.execute(&[], &mut ctx);

        match result {
            CommandResult::Success(Some(msg)) => {
                // Should be sorted
                let lines: Vec<&str> = msg.lines().collect();
                assert_eq!(lines.len(), 3);
                assert_eq!(lines[0], "aws");
                assert_eq!(lines[1], "email");
                assert_eq!(lines[2], "github");
            }
            _ => panic!("Expected success with list"),
        }
    }
}
