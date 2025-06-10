use anyhow::Ok;
use dirtybase_contract::{
    ExtensionManager, ExtensionMigrations, cli_contract::clap::ArgMatches,
    db_contract::base::manager::Manager, prelude::Context,
};

use crate::model::migration::{MigrationRepository, TABLE_NAME};

#[derive(Debug, Clone)]
pub enum MigrateAction {
    Up,
    Down,
    Refresh,
    Reset,
    Unknown,
}

pub struct Migrator {
    migrations: ExtensionMigrations,
}

const LOG_TARGET: &str = "db::migrator";

impl Migrator {
    pub async fn new(context: Option<Context>) -> Self {
        let mut migrations = Vec::new();
        let context = if let Some(ctx) = context {
            ctx
        } else {
            dirtybase_contract::app_contract::global_context().await
        };
        ExtensionManager::extensions(|ext| {
            if let Some(m) = ext.migrations(&context) {
                migrations.extend(m);
            }
        })
        .await;

        Self { migrations }
    }

    pub async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        let batch = chrono::Utc::now().timestamp();
        let repo = self.repo(manager).await;

        for entry in &self.migrations {
            let name = entry.id();
            if !repo.exist(&name).await {
                tracing::debug!(target: LOG_TARGET, "migrating {} up", entry.id());
                entry.up(manager).await?;

                if let Err(e) = repo.create(&name, batch).await {
                    tracing::error!(target: LOG_TARGET,"could not create migration entry: {:?}", e);
                }
            } else {
                tracing::debug!(target: LOG_TARGET, "migration already exist: {:?}", &name);
            }
        }

        Ok(())
    }

    pub async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        let repo = self.repo(manager).await;

        let collection = repo.get_last_batch().await;

        for name in collection.keys() {
            for entry in &self.migrations {
                if entry.id() == name.as_str() {
                    tracing::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
                    entry.down(manager).await?;
                }
            }
        }

        if let Some((_, entry)) = collection.iter().next() {
            repo.delete_batch(entry.batch).await;
        }
        Ok(())
    }

    pub async fn refresh(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        self.down(manager).await?;
        self.up(manager).await
    }

    pub async fn reset(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        loop {
            let repo = self.repo(manager).await;
            let collection = repo.get_last_batch().await;
            if collection.is_empty() {
                break;
            }
            self.down(manager).await?;
        }

        manager.drop_table(TABLE_NAME).await
    }

    async fn repo(&self, manager: &Manager) -> MigrationRepository {
        let repo = MigrationRepository::new(manager.clone());
        if let Err(e) = repo.init().await {
            tracing::error!("could not initialize migrator: {}", e);
        }

        repo
    }
}

impl From<(String, ArgMatches)> for MigrateAction {
    fn from(value: (String, ArgMatches)) -> Self {
        match value.0.as_str() {
            "up" => MigrateAction::Up,
            "down" => MigrateAction::Down,
            "refresh" => MigrateAction::Refresh,
            "reset" => MigrateAction::Reset,
            _ => MigrateAction::Unknown,
        }
    }
}
