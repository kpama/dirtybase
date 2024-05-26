use dirtybase_config::DirtyConfig;
use dirtybase_contract::{cli::CliCommandManager, http::RouterManager};

use crate::{app::command::Commands, run_cli, run_http};

mod migration;

pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&self, _config: &DirtyConfig) {
        // event_handler::setup().await;
    }

    fn migrations(&self) -> dirtybase_contract::ExtensionMigrations {
        migration::setup()
    }

    async fn shutdown(&self) {
        println!("--- main application is shutting down -- ");
    }

    fn register_routes(&self, manager: RouterManager) -> RouterManager {
        manager
    }

    fn register_cli_commands(&self, mut manager: CliCommandManager) -> CliCommandManager {
        // serve command
        let serve = clap::Command::new("serve").about("Start the web application server");
        manager.register(serve, |name, _c, service| {
            Box::pin(async move {
                if let Err(e) = run_http(service.provide().await).await {
                    log::error!("{}", e)
                }
            })
        });

        // migrate command
        let migrate = clap::Command::new("migrate")
            .about("Execute migration")
            .subcommand(clap::Command::new("up").about("Migrate up"))
            .subcommand(clap::Command::new("down").about("Migrate down"))
            .subcommand(clap::Command::new("refresh").about("Resets and migrate all up"))
            .subcommand(clap::Command::new("reset").about("Migrate all down"));
        manager.register(migrate, |name, matches, service| {
            Box::pin(async move {
                let commands: Commands = Commands::from((name, matches));
                if let Err(e) = run_cli(service.provide().await, &commands).await {
                    log::error!("{}", e)
                }
            })
        });

        // Queue command
        let queue = clap::Command::new("queue")
            .about("Process queued jobs")
            .arg(
                clap::Arg::new("name")
                    .short('n')
                    .help("the id of the queue"),
            );
        manager.register(queue, |name, matches, service| {
            Box::pin(async move {
                let commands: Commands = Commands::from((name, matches));
                if let Err(e) = run_cli(service.provide().await, &commands).await {
                    log::error!("{}", e)
                }
            })
        });

        // Handle command
        let queue = clap::Command::new("handle")
            .about("Handle dispatched events")
            .arg(
                clap::Arg::new("cluster")
                    .short('c')
                    .help("the cluster to listen at"),
            );
        manager.register(queue, |name, matches, service| {
            Box::pin(async move {
                let commands: Commands = Commands::from((name, matches));
                if let Err(e) = run_cli(service.provide().await, &commands).await {
                    log::error!("{}", e)
                }
            })
        });

        manager
    }
}
