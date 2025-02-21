use dirtybase_contract::app::Context;
use dirtybase_db::base::manager::Manager;
use tokio_util::sync::CancellationToken;

use crate::core::{
    AppService,
    command::{
        Commands,
        migrator::{MigrateAction, Migrator},
    },
};

pub async fn init(context: Context, command: &Commands) -> anyhow::Result<()> {
    let app = context
        .get::<AppService>()
        .await
        .expect("could not find the app service in the context");
    app.init().await;

    let token = CancellationToken::new();
    let cloned_token = token.clone();

    let handler = tokio::spawn(async move {
        tokio::select! {
            _ = cloned_token.cancelled() => {
                let app_service: AppService = busybody::helpers::provide().await;
                app_service.shutdown().await;
            }
        }
    });

    let the_command = command.clone();
    tokio::spawn(async move {
        // Run only CLI tools
        match the_command {
            Commands::Migrate { action } => {
                let schema_manager = context
                    .get::<Manager>()
                    .await
                    .expect("could not get schema manager");
                let migrator = Migrator::from_app(&app).await;
                log::debug!("executing migration: {:?}", &action);
                match action {
                    MigrateAction::Up => migrator.up(&schema_manager).await,
                    MigrateAction::Down => migrator.down(&schema_manager).await,
                    MigrateAction::Refresh => migrator.refresh(&schema_manager).await,
                    MigrateAction::Reset => migrator.reset(&schema_manager).await,
                }
                token.cancel();
            }
            Commands::Queue { name } => {
                log::debug!("processing jobs on queue: {:?}", name);
                token.cancel();
            }
            Commands::Handle { cluster } => {
                log::debug!("Handling events broadcasted on cluster: {:?}", cluster);
                token.cancel();
            }
            _ => (),
        }
    });

    _ = handler.await;
    Ok(())
}
