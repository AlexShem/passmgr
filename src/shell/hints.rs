//! Command hints for the shell.
//!
//! Provides inline suggestions as the user types.

use rustyline::Context;
use rustyline::hint::Hinter;
use std::sync::Arc;

use crate::shell::command::CommandRegistry;

/// Hinter that provides command usage hints.
pub struct PassmgrHinter {
    /// Registry of available commands.
    registry: Arc<CommandRegistry>,
}

impl PassmgrHinter {
    /// Creates a new hinter.
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self { registry }
    }

    /// Gets a hint for the current input.
    fn get_hint(&self, line: &str) -> Option<String> {
        // Check for trailing space before trimming
        let has_trailing_space = line.ends_with(' ');
        let line = line.trim();

        if line.is_empty() {
            return None;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let command = parts[0];

        // If we're still typing the command, show completion hint
        if parts.len() == 1 && !has_trailing_space {
            let completions = self.registry.completions(command);
            if completions.len() == 1 {
                let completion = &completions[0];
                if completion.starts_with(command) && completion != command {
                    return Some(completion[command.len()..].to_string());
                }
            }
            return None;
        }

        // Show usage hint for the command
        if let Some(cmd) = self.registry.get(command) {
            let arg_count = parts.len() - 1;
            let min_args = cmd.min_args();

            if arg_count < min_args {
                // Show what arguments are needed
                let usage = cmd.usage();
                // Extract the part after the command name
                if let Some(args_part) = usage.strip_prefix(cmd.name()) {
                    let args_part = args_part.trim();
                    if !args_part.is_empty() {
                        // Only show hint if we're missing arguments
                        let hint_parts: Vec<&str> = args_part.split_whitespace().collect();
                        if arg_count < hint_parts.len() {
                            let remaining: Vec<&str> = hint_parts[arg_count..].to_vec();
                            return Some(format!(" {}", remaining.join(" ")));
                        }
                    }
                }
            }
        }

        None
    }
}

impl Hinter for PassmgrHinter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        // Only hint if cursor is at end of line
        if pos < line.len() {
            return None;
        }

        self.get_hint(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::commands::register_all;

    fn setup_hinter() -> PassmgrHinter {
        let mut registry = CommandRegistry::new();
        register_all(&mut registry);
        PassmgrHinter::new(Arc::new(registry))
    }

    #[test]
    fn test_command_completion_hint() {
        let hinter = setup_hinter();

        // "hel" should hint "p" to complete "help"
        let hint = hinter.get_hint("hel");
        assert_eq!(hint, Some("p".to_string()));
    }

    #[test]
    fn test_no_hint_for_complete_command() {
        let hinter = setup_hinter();

        // Full command with no args - show usage hint
        let hint = hinter.get_hint("add ");
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("<name>"));
    }

    #[test]
    fn test_no_hint_when_args_complete() {
        let hinter = setup_hinter();

        // Command with enough args
        let hint = hinter.get_hint("add key secret");
        assert!(hint.is_none());
    }

    #[test]
    fn test_empty_line_no_hint() {
        let hinter = setup_hinter();
        assert!(hinter.get_hint("").is_none());
        assert!(hinter.get_hint("   ").is_none());
    }
}
