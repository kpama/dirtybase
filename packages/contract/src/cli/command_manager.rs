use clap::command;
use futures::future::BoxFuture;
use std::{collections::HashMap, sync::Arc};

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

    pub async fn handle(mut self, service_container: Arc<busybody::ServiceContainer>) {
        let mut command = command!()
            .propagate_version(true)
            .subcommand_required(true)
            .arg_required_else_help(true);

        for cmd in self.commands.into_iter() {
            command = command.subcommand(cmd);
        }

        let mut matches = command.get_matches();

        if let Some((cmd, command)) = matches.remove_subcommand() {
            if let Some(handler) = self.command_handlers.get_mut(&cmd) {
                (handler)(cmd, command, service_container).await;
            }
        }
    }
}
