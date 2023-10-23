use crate::{
    app::{Commands, DirtyBaseAppService},
    migration::{migrator::Migrator, MigrateAction},
};

pub async fn init(app: DirtyBaseAppService, command: &Commands) -> std::io::Result<()> {
    app.init().await;

    // Run only CLI tools
    match &command {
        Commands::Migrate { action } => {
            let migrator = Migrator::from_app(&app).await;
            match *action {
                MigrateAction::Up => migrator.up(&app.schema_manger()).await,
                MigrateAction::Down => migrator.down(&app.schema_manger()).await,
                MigrateAction::Refresh => migrator.refresh(&app.schema_manger()).await,
                MigrateAction::Reset => migrator.reset(&app.schema_manger()).await,
            }
            dbg!("executing migration: {:?}", action);
        }
        Commands::Queue { name } => {
            dbg!("processing jobs on queue: {:?}", name);
        }
        Commands::Handle { cluster } => {
            dbg!("Handling events broadcasted on cluster: {:?}", cluster);
        }
        _ => (),
    }

    Ok(())
}
