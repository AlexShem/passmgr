//! Shell module - rustyline-based interactive shell.
//!
//! This module provides a shell-like interface with:
//! - Command completion
//! - Syntax highlighting
//! - Command history
//! - Command hints

pub mod command;
pub mod commands;
pub mod completer;
pub mod highlighter;
pub mod hints;
pub mod history;

use anyhow::{Result, anyhow};
use rustyline::completion::Completer;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::{
    MatchingBracketValidator, ValidationContext, ValidationResult, Validator,
};
use rustyline::{Context, Editor, Helper};
use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use crate::credentials::Credentials;
use crate::trie::Trie;

use command::{CommandRegistry, CommandResult, ShellContext};
use commands::register_all;
use completer::PassmgrCompleter;
use highlighter::{OutputHighlighter, PassmgrHighlighter};
use hints::PassmgrHinter;
use history::HistoryConfig;

/// The prompt displayed to the user.
const PROMPT: &str = "passmgr> ";

/// Combined helper for rustyline that provides all shell features.
pub struct PassmgrHelper {
    completer: PassmgrCompleter,
    highlighter: PassmgrHighlighter,
    hinter: PassmgrHinter,
    validator: MatchingBracketValidator,
}

impl PassmgrHelper {
    /// Creates a new helper with all shell features.
    pub fn new(registry: Arc<CommandRegistry>, key_trie: Arc<RwLock<Trie>>) -> Self {
        Self {
            completer: PassmgrCompleter::new(Arc::clone(&registry), Arc::clone(&key_trie)),
            highlighter: PassmgrHighlighter::new(Arc::clone(&registry)),
            hinter: PassmgrHinter::new(Arc::clone(&registry)),
            validator: MatchingBracketValidator::new(),
        }
    }
}

// Implement all required traits for PassmgrHelper

impl Completer for PassmgrHelper {
    type Candidate = rustyline::completion::Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Highlighter for PassmgrHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        self.highlighter.highlight_prompt(prompt, default)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        self.highlighter.highlight_hint(hint)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        self.highlighter.highlight_candidate(candidate, completion)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: rustyline::highlight::CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

impl Hinter for PassmgrHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Validator for PassmgrHelper {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
        self.validator.validate(ctx)
    }
}

impl Helper for PassmgrHelper {}

/// Configuration for the shell.
pub struct ShellConfig {
    /// History configuration.
    pub history: HistoryConfig,
    /// Whether to show the welcome message.
    pub show_welcome: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            history: HistoryConfig::default(),
            show_welcome: true,
        }
    }
}

/// The interactive shell.
pub struct Shell {
    /// Command registry.
    registry: Arc<CommandRegistry>,
    /// Key trie for completion (shared with helper).
    key_trie: Arc<RwLock<Trie>>,
    /// Shell configuration.
    config: ShellConfig,
}

impl Shell {
    /// Creates a new shell with default configuration.
    pub fn new() -> Self {
        let mut registry = CommandRegistry::new();
        register_all(&mut registry);

        Self {
            registry: Arc::new(registry),
            key_trie: Arc::new(RwLock::new(Trie::new())),
            config: ShellConfig::default(),
        }
    }

    /// Creates a shell with custom configuration.
    pub fn with_config(config: ShellConfig) -> Self {
        let mut registry = CommandRegistry::new();
        register_all(&mut registry);

        Self {
            registry: Arc::new(registry),
            key_trie: Arc::new(RwLock::new(Trie::new())),
            config,
        }
    }

    /// Initializes the key trie from existing credentials.
    fn init_key_trie(&self, credentials: &Credentials) {
        if let Ok(mut trie) = self.key_trie.write() {
            trie.clear();
            for key in credentials.list() {
                trie.insert(key);
            }
            log::debug!("Initialized key trie with {} entries", trie.len());
        }
    }

    /// Runs the interactive shell with a save callback.
    pub fn run_with_save<F>(&self, credentials: &mut Credentials, mut save_fn: F) -> Result<()>
    where
        F: FnMut(&Credentials) -> Result<()>,
    {
        // Initialize key trie from existing credentials
        self.init_key_trie(credentials);

        // Create the helper
        let helper = PassmgrHelper::new(Arc::clone(&self.registry), Arc::clone(&self.key_trie));

        // Create the editor with our custom helper
        let mut editor: Editor<PassmgrHelper, FileHistory> = Editor::new()?;
        editor.set_helper(Some(helper));

        // Configure history
        editor.set_max_history_size(self.config.history.max_entries)?;

        // Load existing history if the file exists
        if self.config.history.path.exists() {
            if let Err(e) = editor.load_history(&self.config.history.path) {
                log::warn!("Could not load history: {}", e);
            } else {
                log::debug!("Loaded history from {}", self.config.history.path.display());
            }
        }

        if self.config.show_welcome {
            println!("Unlocked. Type 'help' for available commands.");
        }

        log::info!("Shell started");

        // Main REPL loop
        loop {
            match editor.readline(PROMPT) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = editor.add_history_entry(line);

                    // Parse and execute command
                    let mut key_trie_guard = self
                        .key_trie
                        .write()
                        .map_err(|e| anyhow!("Key trie lock poisoned: {}", e))?;
                    let mut ctx = ShellContext::new(credentials, &mut key_trie_guard)
                        .with_registry(&self.registry);

                    let result = self.execute_with_context(line, &mut ctx);
                    let was_modified = ctx.modified;
                    drop(key_trie_guard);

                    match result {
                        CommandResult::Success(Some(msg)) => {
                            println!("{}", msg);
                        }
                        CommandResult::Success(None) => {}
                        CommandResult::Error(msg) => {
                            eprintln!("{}", OutputHighlighter::error(&msg));
                        }
                        CommandResult::Exit => {
                            log::info!("User requested exit");
                            break;
                        }
                        CommandResult::Continue => {}
                    }

                    // Save if credentials were modified
                    if was_modified {
                        if let Err(e) = save_fn(credentials) {
                            eprintln!(
                                "{}",
                                OutputHighlighter::error(&format!("Failed to save: {}", e))
                            );
                            log::error!("Failed to save credentials: {}", e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    log::debug!("Interrupted (Ctrl-C)");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("exit");
                    log::info!("EOF received (Ctrl-D)");
                    break;
                }
                Err(err) => {
                    eprintln!("{}", OutputHighlighter::error(&format!("Error: {}", err)));
                    log::error!("Readline error: {}", err);
                    break;
                }
            }
        }

        // Save history
        if let Some(parent) = self.config.history.path.parent() {
            if !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        if let Err(e) = editor.save_history(&self.config.history.path) {
            log::warn!("Failed to save history: {}", e);
        } else {
            log::debug!("Saved history to {}", self.config.history.path.display());
        }

        log::info!("Shell exited");
        Ok(())
    }

    /// Parses and executes a command line.
    #[allow(unused)]
    fn execute_line(&self, line: &str, credentials: &mut Credentials) -> CommandResult {
        let mut key_trie_guard = self.key_trie.write().unwrap();
        let mut ctx =
            ShellContext::new(credentials, &mut key_trie_guard).with_registry(&self.registry);

        self.execute_with_context(line, &mut ctx)
    }

    /// Executes a command with the given context.
    fn execute_with_context(&self, line: &str, ctx: &mut ShellContext) -> CommandResult {
        // Parse the line into command and arguments
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return CommandResult::Continue;
        }

        let cmd_name = parts[0];
        let args: Vec<&str> = parts[1..].to_vec();

        log::debug!("Executing command: {} with args: {:?}", cmd_name, args);

        // Look up the command
        match self.registry.get(cmd_name) {
            Some(cmd) => {
                let start = std::time::Instant::now();
                let result = cmd.execute(&args, ctx);
                let duration = start.elapsed();
                log::debug!("Command '{}' completed in {:?}", cmd_name, duration);
                result
            }
            None => CommandResult::error(format!(
                "Unknown command: '{}'\nType 'help' to see available commands.",
                cmd_name
            )),
        }
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_creation() {
        let shell = Shell::new();
        assert!(!shell.registry.is_empty());
    }

    #[test]
    fn test_execute_line_unknown_command() {
        let shell = Shell::new();
        let mut credentials = Credentials::new();

        let result = shell.execute_line("unknown_cmd", &mut credentials);
        assert!(matches!(result, CommandResult::Error(_)));
    }

    #[test]
    fn test_execute_line_help() {
        let shell = Shell::new();
        let mut credentials = Credentials::new();

        let result = shell.execute_line("help", &mut credentials);
        assert!(matches!(result, CommandResult::Success(Some(_))));
    }

    #[test]
    fn test_execute_line_quit() {
        let shell = Shell::new();
        let mut credentials = Credentials::new();

        let result = shell.execute_line("quit", &mut credentials);
        assert!(matches!(result, CommandResult::Exit));
    }

    #[test]
    fn test_execute_line_add_and_get() {
        let shell = Shell::new();
        let mut credentials = Credentials::new();

        let result = shell.execute_line("add testkey testsecret", &mut credentials);
        assert!(matches!(result, CommandResult::Success(_)));

        let result = shell.execute_line("get testkey", &mut credentials);
        match result {
            CommandResult::Success(Some(secret)) => assert_eq!(secret, "testsecret"),
            _ => panic!("Expected success with secret"),
        }
    }

    #[test]
    fn test_key_trie_initialization() {
        let shell = Shell::new();
        let mut credentials = Credentials::new();
        credentials
            .add("key1".to_string(), "val1".to_string())
            .unwrap();
        credentials
            .add("key2".to_string(), "val2".to_string())
            .unwrap();

        shell.init_key_trie(&credentials);

        let trie = shell.key_trie.read().unwrap();
        assert!(trie.contains("key1"));
        assert!(trie.contains("key2"));
        assert_eq!(trie.len(), 2);
    }
}
