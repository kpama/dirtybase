use clap::command;
use futures::future::BoxFuture;
use std::{collections::HashMap, sync::Arc};

use crate::ExtensionManager;

type CommandHandler = Box<
    dyn FnMut(String, clap::ArgMatches, Arc<busybody::ServiceContainer>) -> BoxFuture<'static, ()>,
>;
pub struct CliCommandManager {
    command_handlers: HashMap<String, CommandHandler>,
    commands: Vec<clap::Command>,
}

impl Default for CliCommandManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CliCommandManager {
    pub fn new() -> Self {
        Self {
            command_handlers: HashMap::new(),
            commands: Vec::new(),
        }
    }

    pub fn register<H>(&mut self, command: clap::Command, handler: H) -> &mut Self
    where
        H: FnMut(
                String,
                clap::ArgMatches,
                Arc<busybody::ServiceContainer>,
            ) -> BoxFuture<'static, ()>
            + 'static,
    {
        self.command_handlers
            .insert(command.get_name().to_string(), Box::new(handler));

        self.commands.push(command);

        self
    }

    pub async fn handle(self) {
        self.handle_command::<Vec<&str>, &str>(None).await;
    }

    pub async fn handle_command<I, T>(mut self, cmd: Option<I>)
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let mut command = command!()
            .propagate_version(true)
            .subcommand_required(true)
            .arg_required_else_help(true);

        for cmd in self.commands.into_iter() {
            command = command.subcommand(cmd);
        }

        let mut matches = match cmd {
            Some(c) => {
                let mut cmds = vec![String::new()]; // program name
                cmds.extend(c.into_iter().map(|v| v.into()));
                command.get_matches_from(cmds)
            }
            None => command.get_matches(),
        };
        let service_container = busybody::helpers::service_container();

        if let Some((cmd, mut command)) = matches.remove_subcommand() {
            for ext in ExtensionManager::list().read().await.iter() {
                command = ext.on_cli_command(cmd.as_str(), command).await;
            }

            if let Some(handler) = self.command_handlers.get_mut(&cmd) {
                (handler)(cmd, command, service_container).await;
            }
        }
    }
}
