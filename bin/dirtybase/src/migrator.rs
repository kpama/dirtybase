use clap::Subcommand;

use dirtybase_contract::ExtensionMigrations;
use dirtybase_db::base::manager::Manager;
use dirtybase_db_types::{types::ColumnAndValue, TableEntityTrait};

use crate::app::{
    model::migration::{MigrationEntity, MigrationRepository},
    setup_database::setup_migration_table,
    DirtyBaseAppService,
};

#[derive(Subcommand, Debug)]
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
    pub async fn from_app(app: &DirtyBaseAppService) -> Self {
        let mut migrations = Vec::new();
        app.extensions(|ext| {
            migrations.extend(ext.migrations());
        })
        .await;

        Self { migrations }
    }

    pub async fn up(&self, manager: &Manager) {
        let batch = chrono::Utc::now().timestamp();

        for entry in &self.migrations {
            log::debug!(target: LOG_TARGET, "migrating {} up", entry.id());
            // TODO: First check before running the migration

            entry.up(manager).await;
        }
    }

    pub async fn down(&self, manager: &Manager) {
        let repo = self.repo().await;

        for entry in &self.migrations {
            log::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
            // TODO: First check before running the migration
            entry.down(manager).await;
        }
    }

    pub async fn refresh(&self, manager: &Manager) {
        let repo = self.repo().await;
        // Migrate everything down
        for entry in &self.migrations {
            log::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
            entry.down(manager).await
        }

        for entry in &self.migrations {
            log::debug!(target: LOG_TARGET, "migrating {} up", entry.id());
            entry.up(manager).await
        }
    }

    pub async fn reset(&self, manager: &Manager) {
        let repo = self.repo().await;
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
