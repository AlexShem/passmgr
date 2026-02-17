//! Help command implementation.

use crate::shell::command::{Command, CommandResult, ShellContext};

/// Command to display help information.
pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn aliases(&self) -> &[&str] {
        &["h", "?"]
    }

    fn description(&self) -> &str {
        "Display help information"
    }

    fn usage(&self) -> &str {
        "help [command]"
    }

    fn help(&self) -> &str {
        "Display help information about commands.\n\n\
         Without arguments, lists all available commands.\n\
         With a command name, shows detailed help for that command.\n\n\
         Examples:\n  \
           help\n  \
           help add\n  \
           ? get"
    }

    fn execute(&self, args: &[&str], ctx: &mut ShellContext) -> CommandResult {
        let registry = match ctx.registry {
            Some(r) => r,
            None => {
                return CommandResult::error("Help not available (no registry)");
            }
        };

        if args.is_empty() {
            // List all commands
            let mut output = String::from("Available commands:\n\n");

            let mut commands: Vec<_> = registry.commands().collect();
            commands.sort_by_key(|c| c.name());

            for cmd in commands {
                let aliases = cmd.aliases();
                let alias_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", aliases.join(", "))
                };

                output.push_str(&format!(
                    "  {:<12}{} - {}\n",
                    cmd.name(),
                    alias_str,
                    cmd.description()
                ));
            }

            output.push_str("\nType 'help <command>' for detailed help on a specific command.");

            CommandResult::success(output)
        } else {
            // Show help for specific command
            let cmd_name = args[0];

            match registry.get(cmd_name) {
                Some(cmd) => {
                    let aliases = cmd.aliases();
                    let alias_str = if aliases.is_empty() {
                        String::new()
                    } else {
                        format!("\nAliases: {}", aliases.join(", "))
                    };

                    let output = format!(
                        "{}\n\nUsage: {}{}\n\n{}",
                        cmd.name().to_uppercase(),
                        cmd.usage(),
                        alias_str,
                        cmd.help()
                    );

                    CommandResult::success(output)
                }
                None => CommandResult::error(format!(
                    "Unknown command: '{}'\nType 'help' to see available commands.",
                    cmd_name
                )),
            }
        }
    }

    fn completions(&self, arg_index: usize, partial: &str, ctx: &ShellContext) -> Vec<String> {
        if arg_index == 0 {
            // Complete command names for help
            if let Some(registry) = ctx.registry {
                registry.completions(partial)
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    fn min_args(&self) -> usize {
        0
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::Credentials;
    use crate::shell::command::CommandRegistry;
    use crate::shell::commands::register_all;
    use crate::trie::Trie;

    fn setup_registry() -> CommandRegistry {
        let mut registry = CommandRegistry::new();
        register_all(&mut registry);
        registry
    }

    #[test]
    fn test_help_command_list_all() {
        let registry = setup_registry();
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

        let cmd = HelpCommand;
        let result = cmd.execute(&[], &mut ctx);

        match result {
            CommandResult::Success(Some(msg)) => {
                assert!(msg.contains("Available commands"));
                assert!(msg.contains("add"));
                assert!(msg.contains("get"));
                assert!(msg.contains("list"));
                assert!(msg.contains("remove"));
                assert!(msg.contains("help"));
                assert!(msg.contains("quit"));
            }
            _ => panic!("Expected success with help text"),
        }
    }

    #[test]
    fn test_help_command_specific() {
        let registry = setup_registry();
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

        let cmd = HelpCommand;
        let result = cmd.execute(&["add"], &mut ctx);

        match result {
            CommandResult::Success(Some(msg)) => {
                assert!(msg.contains("ADD"));
                assert!(msg.contains("add <name> <secret>"));
            }
            _ => panic!("Expected success with add help"),
        }
    }

    #[test]
    fn test_help_command_unknown() {
        let registry = setup_registry();
        let mut credentials = Credentials::new();
        let mut trie = Trie::new();
        let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

        let cmd = HelpCommand;
        let result = cmd.execute(&["nonexistent"], &mut ctx);

        assert!(matches!(result, CommandResult::Error(_)));
    }
}
