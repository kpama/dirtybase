use clap::command;
use futures::future::BoxFuture;

use crate::{ExtensionManager, app_contract::Context};

use super::CliMiddlewareManager;
use tokio_util::sync::CancellationToken;

type CommandCollection = Vec<(
    clap::Command,
    Box<
        dyn FnMut(
                String,
                clap::ArgMatches,
                Context,
            ) -> BoxFuture<'static, Result<(), anyhow::Error>>
            + Send
            + Sync
            + 'static,
    >,
    Option<Vec<String>>,
)>;

pub struct CliCommandManager {
    commands: CommandCollection,
    middleware_manager: CliMiddlewareManager,
    global_middlewares: Vec<String>,
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
            global_middlewares: Vec::new(),
        }
    }

    pub fn register<H>(&mut self, command: clap::Command, handler: H) -> &mut Self
    where
        H: FnMut(
                String,
                clap::ArgMatches,
                Context,
            ) -> BoxFuture<'static, Result<(), anyhow::Error>>
            + Send
            + Sync
            + 'static,
    {
        if self.global_middlewares.is_empty() {
            self.commands.push((command, Box::new(handler), None));
        } else {
            self.commands.push((
                command,
                Box::new(handler),
                Some(self.global_middlewares.clone()),
            ));
        }

        self
    }

    pub fn apply<H, I>(
        &mut self,
        command: clap::Command,
        handler: H,
        order: impl IntoIterator<Item = I>,
    ) -> &mut Self
    where
        H: FnMut(
                String,
                clap::ArgMatches,
                Context,
            ) -> BoxFuture<'static, Result<(), anyhow::Error>>
            + Send
            + Sync
            + 'static,
        I: Into<String>,
    {
        let mut o = order.into_iter().map(|v| v.into()).collect::<Vec<String>>();

        if self.global_middlewares.is_empty() {
            self.commands.push((command, Box::new(handler), Some(o)));
        } else {
            o.extend(self.global_middlewares.clone());
            self.commands.push((command, Box::new(handler), Some(o)));
        }

        self
    }

    pub(crate) fn set_global_middlware(&mut self, middlewares: Vec<String>) {
        self.global_middlewares = middlewares
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

        let task = tokio::spawn(async move {
            let context = Context::new().await;
            let token = CancellationToken::new();
            let cloned_token = token.clone();
            let cloned_context = context.clone();

            let handler = tokio::spawn(async move {
                tokio::select! {
                    _ = cloned_token.cancelled() => {
                        ExtensionManager::shutdown(&cloned_context).await;
                    }
                }
            });

            if let Some((name, mut command)) = matches.remove_subcommand() {
                for ext in ExtensionManager::list().read().await.iter() {
                    command = ext
                        .on_cli_command(name.as_str(), command, context.clone())
                        .await;
                }

                for (cmd, handler, order) in self.commands.into_iter() {
                    let middleware = self
                        .middleware_manager
                        .apply(handler, order.unwrap_or_default())
                        .await;
                    if name == cmd.get_name() {
                        _ = middleware.send((name, command, context.clone())).await;
                        token.cancel();
                        break;
                    }
                }
            }
            _ = handler.await;
        });

        _ = task.await;
    }
}
