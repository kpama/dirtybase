mod cli_middleware_manager;
mod command_manager;

pub use clap;
pub use cli_middleware_manager::*;
pub use command_manager::*;

use crate::ExtensionManager;

pub mod prelude {
    pub use super::cli_middleware_manager::*;
    pub use super::command_manager::*;
    pub use clap::ArgMatches;
    pub use clap::Command;
    pub use clap::Subcommand;
}

pub async fn setup_cli_command_manager() -> CliCommandManager {
    let lock = ExtensionManager::list().read().await;
    let mut middleware = CliMiddlewareManager::new();

    for ext in lock.iter() {
        middleware = ext.register_cli_middlewares(middleware);
    }
    let mut manager = CliCommandManager::new(middleware);

    for ext in lock.iter() {
        manager = ext.register_cli_commands(manager);
    }

    manager
}

pub async fn run_command<I, T>(command: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<String>,
{
    setup_cli_command_manager()
        .await
        .handle_command(Some(command))
        .await;
    Ok(())
}
pub async fn run() -> anyhow::Result<()> {
    let manager = setup_cli_command_manager().await;
    manager.handle().await;
    Ok(())
}
