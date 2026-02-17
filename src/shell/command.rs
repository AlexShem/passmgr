//! Command trait and registry for the shell.
//!
//! This module defines the command system that replaces clap for the REPL.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::credentials::Credentials;
use crate::trie::Trie;

/// Result of executing a command.
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Command executed successfully with optional message.
    Success(Option<String>),
    /// Command failed with error message.
    Error(String),
    /// Signal to exit the shell.
    Exit,
    /// Continue without output.
    Continue,
}

impl CommandResult {
    /// Creates a success result with a message.
    pub fn success(msg: impl Into<String>) -> Self {
        CommandResult::Success(Some(msg.into()))
    }

    /// Creates a success result without a message.
    #[allow(unused)]
    pub fn ok() -> Self {
        CommandResult::Success(None)
    }

    /// Creates an error result.
    pub fn error(msg: impl Into<String>) -> Self {
        CommandResult::Error(msg.into())
    }
}

/// Context available to commands during execution.
pub struct ShellContext<'a> {
    /// Mutable reference to credentials.
    pub credentials: &'a mut Credentials,
    /// Flag indicating if credentials have been modified.
    pub modified: bool,
    /// Reference to the command registry for help command.
    pub registry: Option<&'a CommandRegistry>,
    /// The key trie for completions (updated on credential changes).
    pub key_trie: &'a mut Trie,
}

impl<'a> ShellContext<'a> {
    /// Creates a new shell context.
    pub fn new(credentials: &'a mut Credentials, key_trie: &'a mut Trie) -> Self {
        Self {
            credentials,
            modified: false,
            registry: None,
            key_trie,
        }
    }

    /// Sets the registry reference for help command.
    pub fn with_registry(mut self, registry: &'a CommandRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Marks credentials as modified.
    pub fn mark_modified(&mut self) {
        self.modified = true;
    }
}

/// A command that can be executed in the shell.
pub trait Command: Send + Sync {
    /// Returns the primary name of the command.
    fn name(&self) -> &str;

    /// Returns command aliases (alternative names).
    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// Returns a short description of the command.
    fn description(&self) -> &str;

    /// Returns usage information (e.g., "add <name> <secret>").
    fn usage(&self) -> &str;

    /// Returns detailed help text.
    fn help(&self) -> &str {
        self.description()
    }

    /// Executes the command with the given arguments.
    fn execute(&self, args: &[&str], ctx: &mut ShellContext) -> CommandResult;

    /// Returns completions for the command's arguments.
    ///
    /// `arg_index` is the 0-based index of the argument being completed.
    /// `partial` is the partial text entered for that argument.
    #[allow(unused)]
    fn completions(&self, _arg_index: usize, _partial: &str, _ctx: &ShellContext) -> Vec<String> {
        vec![]
    }

    /// Returns the minimum number of required arguments.
    fn min_args(&self) -> usize {
        0
    }

    /// Returns the maximum number of arguments (None = unlimited).
    #[allow(unused)]
    fn max_args(&self) -> Option<usize> {
        None
    }
}

impl fmt::Debug for dyn Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name())
            .field("description", &self.description())
            .finish()
    }
}

/// Registry of all available commands.
pub struct CommandRegistry {
    /// Commands indexed by their primary name.
    commands: HashMap<String, Arc<dyn Command>>,
    /// Alias to primary name mapping.
    aliases: HashMap<String, String>,
    /// Trie for command name completion.
    command_trie: Trie,
}

impl CommandRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            command_trie: Trie::new(),
        }
    }

    /// Registers a command.
    pub fn register(&mut self, command: Arc<dyn Command>) {
        let name = command.name().to_string();

        // Add to trie
        self.command_trie.insert(&name);

        // Register aliases
        for alias in command.aliases() {
            self.aliases.insert(alias.to_string(), name.clone());
            self.command_trie.insert(alias);
        }

        // Store command
        self.commands.insert(name, command);
    }

    /// Looks up a command by name or alias.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Command>> {
        // Try direct lookup first
        if let Some(cmd) = self.commands.get(name) {
            return Some(Arc::clone(cmd));
        }

        // Try alias lookup
        if let Some(primary) = self.aliases.get(name) {
            return self.commands.get(primary).map(Arc::clone);
        }

        None
    }

    /// Returns all registered commands.
    pub fn commands(&self) -> impl Iterator<Item = &Arc<dyn Command>> {
        self.commands.values()
    }

    /// Returns all command names (primary names only).
    #[allow(unused)]
    pub fn names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// Returns command name completions for the given prefix.
    pub fn completions(&self, prefix: &str) -> Vec<String> {
        self.command_trie.completions(prefix)
    }

    /// Returns the number of registered commands.
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns true if no commands are registered.
    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand;

    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test"
        }

        fn aliases(&self) -> &[&str] {
            &["t", "tst"]
        }

        fn description(&self) -> &str {
            "A test command"
        }

        fn usage(&self) -> &str {
            "test [args...]"
        }

        fn execute(&self, args: &[&str], _ctx: &mut ShellContext) -> CommandResult {
            if args.is_empty() {
                CommandResult::ok()
            } else {
                CommandResult::success(format!("Args: {:?}", args))
            }
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = CommandRegistry::new();
        registry.register(Arc::new(TestCommand));

        assert!(registry.get("test").is_some());
        assert!(registry.get("t").is_some());
        assert!(registry.get("tst").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_registry_completions() {
        let mut registry = CommandRegistry::new();
        registry.register(Arc::new(TestCommand));

        let completions = registry.completions("te");
        assert!(completions.contains(&"test".to_string()));

        let completions = registry.completions("t");
        assert!(completions.contains(&"test".to_string()));
        assert!(completions.contains(&"tst".to_string()));
    }

    #[test]
    fn test_command_result() {
        let success = CommandResult::success("done");
        assert!(matches!(success, CommandResult::Success(Some(_))));

        let ok = CommandResult::ok();
        assert!(matches!(ok, CommandResult::Success(None)));

        let error = CommandResult::error("failed");
        assert!(matches!(error, CommandResult::Error(_)));
    }
}
