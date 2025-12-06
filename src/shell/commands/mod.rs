//! Individual command implementations.

mod add;
mod get;
mod help;
mod list;
mod quit;
mod remove;

pub use add::AddCommand;
pub use get::GetCommand;
pub use help::HelpCommand;
pub use list::ListCommand;
pub use quit::QuitCommand;
pub use remove::RemoveCommand;

use std::sync::Arc;

use super::command::CommandRegistry;

/// Registers all built-in commands with the registry.
pub fn register_all(registry: &mut CommandRegistry) {
    registry.register(Arc::new(AddCommand));
    registry.register(Arc::new(GetCommand));
    registry.register(Arc::new(RemoveCommand));
    registry.register(Arc::new(ListCommand));
    registry.register(Arc::new(HelpCommand));
    registry.register(Arc::new(QuitCommand));
}
