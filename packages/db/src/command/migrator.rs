use anyhow::Ok;
use dirtybase_contract::{
    ExtensionManager,
    cli_contract::clap::ArgMatches,
    db_contract::{base::manager::Manager, migration::Migration},
    prelude::Context,
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
    context: Context,
}

const LOG_TARGET: &str = "db::migrator";

impl Migrator {
    pub async fn new(context: Option<Context>) -> Self {
        let context = if let Some(ctx) = context {
            ctx
        } else {
            dirtybase_contract::app_contract::global_context().await
        };

        Self { context }
    }

    pub async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        let batch = chrono::Utc::now().timestamp();
        let repo = self.repo(manager).await;

        let migrations = self.migrations().await;

        manager.transaction(|trans| async move {
            for entry in &migrations {
            let name = entry.id();
            if !repo.exist(&name).await {
                    tracing::debug!(target: LOG_TARGET, "migrating {} up", &name);
                    if let Err(e) = entry.up(&trans).await {
                        let collection = repo.get_batch(batch).await;
                        for name in collection.keys() {
                            for entry in &migrations {
                                if entry.id() == name.as_str() {
                                    tracing::trace!(target: LOG_TARGET, "reverting migration: {}", entry.id());
                                    entry.down(&trans).await?
                                }
                            }
                        }
                        repo.delete_batch(batch).await;
                        return Err(e);
                    }

                    if let Err(e) = repo.create(&name, batch).await {
                        tracing::error!(target: LOG_TARGET,"could not create migration entry: {:?}", &e); 
                        entry.down(&trans).await?;
                        return Err(e);
                    }
                } else {
                   tracing::debug!(target: LOG_TARGET, "migration already exist: {:?}", &name);
                }
            }
            return Ok(())
        }).await
    }

    pub async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        let repo = self.repo(manager).await;

        let collection = repo.get_last_batch().await;
        let migrations = self.migrations().await;
        manager
            .transaction(|trans| async move {
                for name in collection.keys() {
                    for entry in &migrations {
                        if entry.id() == name.as_str() {
                            tracing::debug!(target: LOG_TARGET, "migrating {} down", entry.id());
                            entry.down(&trans).await?;
                        }
                    }
                }

                if let Some((name, _)) = collection.iter().next() {
                    _ = repo.delete(&name).await;
                }
                Ok(())
            })
            .await
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
        if let Err(e) = repo.init().await
            && e.to_string() != "migrations already exist"
        {
            tracing::error!("could not initialize migrator: {}", e);
            panic!("could not initialize migrator: {}", e);
        }

        repo
    }

    async fn migrations(&self) -> Vec<Box<dyn Migration>> {
        let mut migrations = Vec::with_capacity(100);
        ExtensionManager::extensions(|ext| {
            if let Some(m) = ext.migrations(&self.context) {
                migrations.extend(m);
            }
        })
        .await;
        migrations
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
