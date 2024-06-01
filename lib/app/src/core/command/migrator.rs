use clap::{ArgMatches, Subcommand};

use dirtybase_contract::ExtensionMigrations;
use dirtybase_db::base::manager::Manager;

use crate::core::model::migration::MigrationRepository;
use crate::core::AppService;

#[derive(Subcommand, Debug, Clone)]
pub enum MigrateAction {
    /// Migrate up
    Up,
    /// Migrate down
    Down,
    /// Resets and migrate all up
    Refresh,
    /// Migrate all down
    Reset,
}

pub struct Migrator {
    migrations: ExtensionMigrations,
}

const LOG_TARGET: &str = "migrator";

impl Migrator {
    pub async fn from_app(app: &AppService) -> Self {
        let mut migrations = Vec::new();
        app.extensions(|ext| {
            migrations.extend(ext.migrations());
        })
        .await;

        Self { migrations }
    }

    pub async fn up(&self, manager: &Manager) {
        let batch = chrono::Utc::now().timestamp();
        let repo = self.repo().await;

        for entry in &self.migrations {
            let name = entry.id();
            if !repo.exist(&name).await {
                log::debug!(target: LOG_TARGET, "migrating {} up", entry.id());
                entry.up(manager).await;

                if let Err(e) = repo.create(&name, batch).await {
                    log::error!(target: LOG_TARGET,"could not create migration entry: {:?}", e);
                }
            } else {
                log::debug!(target: LOG_TARGET, "migration already exist: {:?}", &name);
            }
        }
    }

    pub async fn down(&self, manager: &Manager) {
        let repo = self.repo().await;

        let collection = repo.get_last_batch().await;

        for (name, _) in collection {
            for entry in &self.migrations {
                if entry.id() == name {
                    log::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
                    entry.down(manager).await;
                }
            }
        }
    }

    pub async fn refresh(&self, manager: &Manager) {
        // Migrate everything down
        for entry in &self.migrations {
            log::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
            entry.down(manager).await
        }

        self.up(manager).await
    }

    pub async fn reset(&self, manager: &Manager) {
        let _repo = self.repo().await;
        for entry in &self.migrations {
            log::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
            entry.down(manager).await
        }
    }

    async fn repo(&self) -> MigrationRepository {
        let repo = busybody::helpers::provide::<MigrationRepository>().await;
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
            v @ _ => panic!("{} is not a migration action", v),
        }
    }
}
