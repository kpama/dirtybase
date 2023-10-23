use dirtybase_contract::ExtensionMigrations;
use dirtybase_db::base::manager::Manager;

use crate::app::DirtyBaseAppService;

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
        for entry in &self.migrations {
            log::debug!(target: LOG_TARGET, "migrating {} up", entry.id());
            // TODO: First check before running the migration
            entry.up(manager).await
        }
    }

    pub async fn down(&self, manager: &Manager) {
        for entry in &self.migrations {
            log::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
            // TODO: First check before running the migration
            entry.down(manager).await
        }
    }

    pub async fn refresh(&self, manager: &Manager) {
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
        for entry in &self.migrations {
            log::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
            entry.down(manager).await
        }
    }
}
