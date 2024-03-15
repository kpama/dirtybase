use tokio_util::sync::CancellationToken;

use crate::app::{
    command::{
        migrator::{MigrateAction, Migrator},
        Commands,
    },
    AppService,
};

pub async fn init(app: AppService, command: &Commands) -> anyhow::Result<()> {
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
                let migrator = Migrator::from_app(&app).await;
                match action {
                    MigrateAction::Up => migrator.up(&app.schema_manger()).await,
                    MigrateAction::Down => migrator.down(&app.schema_manger()).await,
                    MigrateAction::Refresh => migrator.refresh(&app.schema_manger()).await,
                    MigrateAction::Reset => migrator.reset(&app.schema_manger()).await,
                }
                log::debug!("executing migration: {:?}", action);
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
