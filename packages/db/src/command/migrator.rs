// use clap::{ArgMatches, Subcommand};

use dirtybase_contract::{
    ExtensionManager, ExtensionMigrations, cli::clap, db::base::manager::Manager,
    prelude::ArgMatches,
};

use crate::model::migration::MigrationRepository;

#[derive(Debug, Clone)]
pub enum MigrateAction {
    Up = 0,
    Down = 1,
    Refresh = 2,
    Reset = 3,
    Unknown = 4,
}

pub struct Migrator {
    migrations: ExtensionMigrations,
}

const LOG_TARGET: &str = "db::migrator";

impl Migrator {
    pub async fn new() -> Self {
        let mut migrations = Vec::new();
        let context = dirtybase_contract::app::global_context().await;
        ExtensionManager::extensions(|ext| {
            if let Some(m) = ext.migrations(&context) {
                migrations.extend(m);
            }
        })
        .await;

        Self { migrations }
    }

    pub async fn up(&self, manager: &Manager) {
        let batch = chrono::Utc::now().timestamp();
        let repo = self.repo(manager).await;

        for entry in &self.migrations {
            let name = entry.id();
            if !repo.exist(&name).await {
                tracing::debug!(target: LOG_TARGET, "migrating {} up", entry.id());
                entry.up(manager).await;

                if let Err(e) = repo.create(&name, batch).await {
                    tracing::error!(target: LOG_TARGET,"could not create migration entry: {:?}", e);
                }
            } else {
                tracing::debug!(target: LOG_TARGET, "migration already exist: {:?}", &name);
            }
        }
    }

    pub async fn down(&self, manager: &Manager) {
        let repo = self.repo(manager).await;

        let collection = repo.get_last_batch().await;

        for name in collection.keys() {
            for entry in &self.migrations {
                if entry.id() == *name {
                    tracing::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
                    entry.down(manager).await;
                }
            }
        }

        if let Some((_, entry)) = collection.iter().next() {
            repo.delete_batch(entry.batch).await;
        }
    }

    pub async fn refresh(&self, manager: &Manager) {
        // Migrate everything down
        for entry in &self.migrations {
            tracing::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
            entry.down(manager).await
        }

        self.up(manager).await
    }

    pub async fn reset(&self, manager: &Manager) {
        let _repo = self.repo(manager).await;
        for entry in &self.migrations {
            tracing::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
            entry.down(manager).await
        }
    }

    async fn repo(&self, manager: &Manager) -> MigrationRepository {
        let repo = MigrationRepository::new(manager.clone());
        repo.init().await;

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
            v => {
                tracing::error!("{} is not a migration action", v);
                MigrateAction::Unknown
            }
        }
    }
}
