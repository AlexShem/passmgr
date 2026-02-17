//! Trie-based autocomplete for rustyline.
//!
//! Provides command and credential key completion.

use rustyline::Context;
use rustyline::completion::{Completer, Pair};
use std::sync::{Arc, RwLock};

use crate::shell::command::CommandRegistry;
use crate::trie::Trie;

/// Completer that handles both command and argument completion.
pub struct PassmgrCompleter {
    /// Registry of available commands.
    registry: Arc<CommandRegistry>,
    /// Trie containing credential keys (updated dynamically).
    key_trie: Arc<RwLock<Trie>>,
}

impl PassmgrCompleter {
    /// Creates a new completer.
    pub fn new(registry: Arc<CommandRegistry>, key_trie: Arc<RwLock<Trie>>) -> Self {
        Self { registry, key_trie }
    }

    /// Gets completions for a command name.
    fn complete_command(&self, partial: &str) -> Vec<Pair> {
        self.registry
            .completions(partial)
            .into_iter()
            .map(|s| Pair {
                display: s.clone(),
                replacement: s,
            })
            .collect()
    }

    /// Gets completions for a credential key.
    fn complete_key(&self, partial: &str) -> Vec<Pair> {
        match self.key_trie.read() {
            Ok(trie) => trie
                .completions(partial)
                .into_iter()
                .map(|s| Pair {
                    display: s.clone(),
                    replacement: s,
                })
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Parses the input line to determine completion context.
    fn parse_context<'a>(&self, line: &'a str, pos: usize) -> CompletionContext<'a> {
        let line_to_pos = &line[..pos];
        let parts: Vec<&str> = line_to_pos.split_whitespace().collect();

        if parts.is_empty() {
            return CompletionContext::Command { partial: "" };
        }

        // Check if we're at the start of a new word (after whitespace)
        let ends_with_space = line_to_pos.ends_with(' ');

        if parts.len() == 1 && !ends_with_space {
            // Still typing the command
            return CompletionContext::Command { partial: parts[0] };
        }

        let command = parts[0];
        let arg_index = if ends_with_space {
            parts.len() - 1
        } else {
            parts.len() - 2
        };
        let partial = if ends_with_space {
            ""
        } else {
            parts.last().unwrap_or(&"")
        };

        CompletionContext::Argument {
            command,
            arg_index,
            partial,
        }
    }
}

/// Context for completion - are we completing a command or an argument?
enum CompletionContext<'a> {
    Command {
        partial: &'a str,
    },
    Argument {
        command: &'a str,
        arg_index: usize,
        partial: &'a str,
    },
}

impl Completer for PassmgrCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let context = self.parse_context(line, pos);

        match context {
            CompletionContext::Command { partial } => {
                let start = pos - partial.len();
                let completions = self.complete_command(partial);
                Ok((start, completions))
            }
            CompletionContext::Argument {
                command,
                arg_index,
                partial,
            } => {
                // Determine what kind of completions based on command
                let completions = match command {
                    // Commands that complete credential keys
                    "get" | "g" | "show" | "remove" | "rm" | "delete" | "del" => {
                        if arg_index == 0 {
                            self.complete_key(partial)
                        } else {
                            vec![]
                        }
                    }
                    // Help command completes command names
                    "help" | "h" | "?" => {
                        if arg_index == 0 {
                            self.complete_command(partial)
                        } else {
                            vec![]
                        }
                    }
                    // Add command doesn't complete (new names)
                    "add" | "a" | "new" | "set" => vec![],
                    // List and quit have no arguments
                    "list" | "ls" | "l" | "quit" | "exit" | "q" => vec![],
                    // Unknown command - no completions
                    _ => vec![],
                };

                let start = pos - partial.len();
                Ok((start, completions))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::commands::register_all;

    fn setup_completer() -> PassmgrCompleter {
        let mut registry = CommandRegistry::new();
        register_all(&mut registry);

        let mut key_trie = Trie::new();
        key_trie.insert("github");
        key_trie.insert("gitlab");
        key_trie.insert("email");
        key_trie.insert("aws");

        PassmgrCompleter::new(Arc::new(registry), Arc::new(RwLock::new(key_trie)))
    }

    #[test]
    fn test_complete_command_partial() {
        let completer = setup_completer();
        let completions = completer.complete_command("ge");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].display, "get");
    }

    #[test]
    fn test_complete_command_empty() {
        let completer = setup_completer();
        let completions = completer.complete_command("");

        // Should return all commands
        assert!(completions.len() >= 6); // add, get, remove, list, help, quit
    }

    #[test]
    fn test_complete_key_partial() {
        let completer = setup_completer();
        let completions = completer.complete_key("gi");

        assert_eq!(completions.len(), 2);
        let displays: Vec<&str> = completions.iter().map(|p| p.display.as_str()).collect();
        assert!(displays.contains(&"github"));
        assert!(displays.contains(&"gitlab"));
    }

    #[test]
    fn test_parse_context_command() {
        let completer = setup_completer();

        let ctx = completer.parse_context("ge", 2);
        assert!(matches!(ctx, CompletionContext::Command { partial: "ge" }));

        let ctx = completer.parse_context("", 0);
        assert!(matches!(ctx, CompletionContext::Command { partial: "" }));
    }

    #[test]
    fn test_parse_context_argument() {
        let completer = setup_completer();

        let ctx = completer.parse_context("get gi", 6);
        assert!(matches!(
            ctx,
            CompletionContext::Argument {
                command: "get",
                arg_index: 0,
                partial: "gi"
            }
        ));

        let ctx = completer.parse_context("get ", 4);
        assert!(matches!(
            ctx,
            CompletionContext::Argument {
                command: "get",
                arg_index: 0,
                partial: ""
            }
        ));
    }
}
