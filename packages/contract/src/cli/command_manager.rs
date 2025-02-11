use clap::command;
use futures::future::BoxFuture;

use crate::{app::Context, ExtensionManager};

use super::CliMiddlewareManager;

pub struct CliCommandManager {
    commands: Vec<(
        clap::Command,
        Box<
            dyn FnMut(String, clap::ArgMatches, Context) -> BoxFuture<'static, ()>
                + Send
                + Sync
                + 'static,
        >,
        Option<Vec<String>>,
    )>,
    middleware_manager: CliMiddlewareManager,
}

impl Default for CliCommandManager {
    fn default() -> Self {
        Self::new(CliMiddlewareManager::new())
    }
}

impl CliCommandManager {
    pub fn new(middleware_manager: CliMiddlewareManager) -> Self {
        Self {
            commands: Vec::new(),
            middleware_manager,
        }
    }

    pub fn register<H>(&mut self, command: clap::Command, handler: H) -> &mut Self
    where
        H: FnMut(String, clap::ArgMatches, Context) -> BoxFuture<'static, ()>
            + Send
            + Sync
            + 'static,
    {
        self.commands.push((command, Box::new(handler), None));

        self
    }

    pub fn apply<H, I>(
        &mut self,
        command: clap::Command,
        handler: H,
        order: impl IntoIterator<Item = I>,
    ) -> &mut Self
    where
        H: FnMut(String, clap::ArgMatches, Context) -> BoxFuture<'static, ()>
            + Send
            + Sync
            + 'static,
        I: Into<String>,
    {
        let o = order.into_iter().map(|v| v.into()).collect::<Vec<String>>();
        self.commands.push((command, Box::new(handler), Some(o)));

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

        for (cmd, _, _) in self.commands.iter() {
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

        let context = Context::default();
        if let Some((name, mut command)) = matches.remove_subcommand() {
            for ext in ExtensionManager::list().read().await.iter() {
                command = ext
                    .on_cli_command(name.as_str(), command, context.clone())
                    .await;
            }

            for (cmd, handler, order) in self.commands.into_iter() {
                let middleware = self
                    .middleware_manager
                    .apply(handler, order.unwrap_or_default());
                if name == cmd.get_name() {
                    middleware.send((name, command, context)).await;
                    break;
                }
            }
        }
    }
}
