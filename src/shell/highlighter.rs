//! Syntax and semantic highlighting for the shell.
//!
//! Provides colorized output for commands, arguments, and results.

use rustyline::highlight::{CmdKind, Highlighter};
use std::borrow::Cow;
use std::sync::Arc;

use crate::shell::command::CommandRegistry;

/// ANSI color codes for highlighting.
pub mod colors {
    /// Reset all formatting.
    pub const RESET: &str = "\x1b[0m";
    /// Bold text.
    pub const BOLD: &str = "\x1b[1m";
    /// Dim text.
    pub const DIM: &str = "\x1b[2m";
    /// Italic text.
    #[allow(unused)]
    pub const ITALIC: &str = "\x1b[3m";
    /// Underline text.
    #[allow(unused)]
    pub const UNDERLINE: &str = "\x1b[4m";

    /// Red foreground.
    pub const RED: &str = "\x1b[31m";
    /// Green foreground.
    #[allow(unused)]
    pub const GREEN: &str = "\x1b[32m";
    /// Yellow foreground.
    pub const YELLOW: &str = "\x1b[33m";
    /// Blue foreground.
    #[allow(unused)]
    pub const BLUE: &str = "\x1b[34m";
    /// Magenta foreground.
    pub const MAGENTA: &str = "\x1b[35m";
    /// Cyan foreground.
    pub const CYAN: &str = "\x1b[36m";
    /// White foreground.
    pub const WHITE: &str = "\x1b[37m";

    /// Bright red foreground.
    pub const BRIGHT_RED: &str = "\x1b[91m";
    /// Bright green foreground.
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    /// Bright yellow foreground.
    #[allow(unused)]
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    /// Bright cyan foreground.
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
}

/// Highlighter for shell input with syntax coloring.
pub struct PassmgrHighlighter {
    /// Registry to check for valid commands.
    registry: Arc<CommandRegistry>,
}

impl PassmgrHighlighter {
    /// Creates a new highlighter.
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self { registry }
    }

    /// Highlights a line of input.
    fn highlight_line(&self, line: &str) -> String {
        if line.trim().is_empty() {
            return line.to_string();
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return line.to_string();
        }

        let command = parts[0];
        let is_valid_command = self.registry.get(command).is_some();

        let mut result = String::new();

        // Find the leading whitespace
        let leading_ws = &line[..line.len() - line.trim_start().len()];
        result.push_str(leading_ws);

        // Highlight the command
        if is_valid_command {
            result.push_str(colors::BOLD);
            result.push_str(colors::CYAN);
            result.push_str(command);
            result.push_str(colors::RESET);
        } else {
            // Invalid command - show in red
            result.push_str(colors::RED);
            result.push_str(command);
            result.push_str(colors::RESET);
        }

        // Find where arguments start
        let cmd_end = line.find(command).unwrap_or(0) + command.len();
        let rest = &line[cmd_end..];

        if !rest.is_empty() {
            // Highlight arguments based on command type
            let highlighted_args = self.highlight_arguments(command, rest);
            result.push_str(&highlighted_args);
        }

        result
    }

    /// Highlights command arguments with appropriate colors.
    fn highlight_arguments(&self, command: &str, args_str: &str) -> String {
        let mut result = String::new();
        let parts: Vec<&str> = args_str.split_whitespace().collect();

        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            // Find this part in the string and preserve whitespace
            let part_start = args_str[pos..].find(part).unwrap_or(0) + pos;
            let whitespace = &args_str[pos..part_start];
            result.push_str(whitespace);

            // Color based on command and argument position
            let color = match command {
                "add" | "a" | "new" | "set" => {
                    if i == 0 {
                        colors::MAGENTA // Key name
                    } else {
                        colors::DIM // Secret (dimmed for privacy)
                    }
                }
                "get" | "g" | "show" | "remove" | "rm" | "delete" | "del" => {
                    colors::MAGENTA // Key name
                }
                "help" | "h" | "?" => {
                    colors::YELLOW // Command name for help
                }
                _ => colors::WHITE,
            };

            result.push_str(color);
            result.push_str(part);
            result.push_str(colors::RESET);

            pos = part_start + part.len();
        }

        // Add any trailing whitespace
        if pos < args_str.len() {
            result.push_str(&args_str[pos..]);
        }

        result
    }
}

impl Highlighter for PassmgrHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(self.highlight_line(line))
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        // Highlight the prompt with a nice color
        Cow::Owned(format!(
            "{}{}{}{}",
            colors::BOLD,
            colors::BRIGHT_GREEN,
            prompt,
            colors::RESET
        ))
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        // Dim the hint text
        Cow::Owned(format!("{}{}{}", colors::DIM, hint, colors::RESET))
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        // Highlight completion candidates
        Cow::Owned(format!(
            "{}{}{}",
            colors::BRIGHT_CYAN,
            candidate,
            colors::RESET
        ))
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        // Return true to enable highlighting
        true
    }
}

/// Utilities for semantic highlighting in output.
pub struct OutputHighlighter;

impl OutputHighlighter {
    /// Formats a success message.
    #[allow(unused)]
    pub fn success(msg: &str) -> String {
        format!("{}{}{}", colors::GREEN, msg, colors::RESET)
    }

    /// Formats an error message.
    pub fn error(msg: &str) -> String {
        format!("{}{}{}", colors::BRIGHT_RED, msg, colors::RESET)
    }

    /// Formats a warning message.
    #[allow(unused)]
    pub fn warning(msg: &str) -> String {
        format!("{}{}{}", colors::YELLOW, msg, colors::RESET)
    }

    /// Formats a key/credential name.
    #[allow(unused)]
    pub fn key(name: &str) -> String {
        format!("{}{}{}", colors::MAGENTA, name, colors::RESET)
    }

    /// Formats a secret (dimmed for less visibility).
    #[allow(unused)]
    pub fn secret(secret: &str) -> String {
        format!("{}{}{}", colors::DIM, secret, colors::RESET)
    }

    /// Formats a command name.
    #[allow(unused)]
    pub fn command(cmd: &str) -> String {
        format!("{}{}{}{}", colors::BOLD, colors::CYAN, cmd, colors::RESET)
    }

    /// Formats informational text.
    #[allow(unused)]
    pub fn info(msg: &str) -> String {
        format!("{}{}{}", colors::BLUE, msg, colors::RESET)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::commands::register_all;

    fn setup_highlighter() -> PassmgrHighlighter {
        let mut registry = CommandRegistry::new();
        register_all(&mut registry);
        PassmgrHighlighter::new(Arc::new(registry))
    }

    #[test]
    fn test_highlight_valid_command() {
        let highlighter = setup_highlighter();
        let result = highlighter.highlight_line("add");

        assert!(result.contains(colors::CYAN));
        assert!(result.contains(colors::BOLD));
        assert!(result.contains("add"));
    }

    #[test]
    fn test_highlight_invalid_command() {
        let highlighter = setup_highlighter();
        let result = highlighter.highlight_line("invalid");

        assert!(result.contains(colors::RED));
        assert!(result.contains("invalid"));
    }

    #[test]
    fn test_highlight_with_arguments() {
        let highlighter = setup_highlighter();
        let result = highlighter.highlight_line("add mykey mysecret");

        assert!(result.contains(colors::CYAN)); // command
        assert!(result.contains(colors::MAGENTA)); // key
        assert!(result.contains(colors::DIM)); // secret
    }

    #[test]
    fn test_output_highlighter_success() {
        let result = OutputHighlighter::success("Done!");
        assert!(result.contains(colors::GREEN));
        assert!(result.contains("Done!"));
    }

    #[test]
    fn test_output_highlighter_error() {
        let result = OutputHighlighter::error("Failed!");
        assert!(result.contains(colors::BRIGHT_RED));
        assert!(result.contains("Failed!"));
    }

    #[test]
    fn test_empty_line() {
        let highlighter = setup_highlighter();
        let result = highlighter.highlight_line("");
        assert_eq!(result, "");

        let result = highlighter.highlight_line("   ");
        assert_eq!(result, "   ");
    }
}
