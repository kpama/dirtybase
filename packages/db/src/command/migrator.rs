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
    List,
    Refresh,
    Reset,
}

pub struct Migrator {
    context: Context,
}

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
        let context = self.context.clone();

        manager
            .transaction(|trans| async move {
                let ctx = context.clone();
                for entry in &migrations {
                    let name = entry.id();
                    if !repo.exist(&name).await {
                        tracing::debug!("migrating {} up", &name);
                        if let Err(e) = entry.up(&trans, &ctx).await {
                            tracing::debug!("reverting migration: {}", entry.id());
                            entry.down(&trans, &ctx).await?;
                            let collection = repo.get_batch(batch).await;
                            for name in collection.keys() {
                                for entry in &migrations {
                                    if entry.id() != name.as_str() {
                                        tracing::debug!("reverting migration: {}", entry.id());
                                        entry.down(&trans, &ctx).await?
                                    }
                                }
                            }
                            repo.delete_batch(batch).await;
                            return Err(e);
                        }

                        if let Err(e) = repo.create(&name, batch).await {
                            tracing::error!("could not create migration entry: {:?}", &e);
                            entry.down(&trans, &ctx).await?;
                            return Err(e);
                        }
                    } else {
                        tracing::debug!("migration already exist: {:?}", &name);
                    }
                }
                return Ok(());
            })
            .await
    }

    pub async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        let repo = self.repo(manager).await;

        let collection = repo.get_last_batch().await;
        let migrations = self.migrations().await;
        let ctx = self.context.clone();
        manager
            .transaction(|trans| async move {
                for name in collection.keys() {
                    for entry in &migrations {
                        if entry.id() == name.as_str() {
                            tracing::debug!("migrating {} down", entry.id());
                            entry.down(&trans, &ctx).await?;
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
        let repo = self.repo(manager).await;
        loop {
            let collection = repo.get_last_batch().await;
            if collection.is_empty() {
                break;
            }
            self.down(manager).await?;
        }
        manager.drop_table(TABLE_NAME).await?;
        self.up(manager).await
    }

    pub async fn reset(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        let repo = self.repo(manager).await;
        loop {
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

    pub async fn list(&self) -> Vec<Box<dyn Migration>> {
        let mut migrations = Vec::with_capacity(110);
        for ext in ExtensionManager::list().read().await.iter() {
            if let Some(list) = ext.migrations(&self.context).await {
                for m in list {
                    migrations.push(m);
                }
            }
        }

        migrations.reverse();
        migrations
    }

    async fn migrations(&self) -> Vec<Box<dyn Migration>> {
        let mut migrations = Vec::with_capacity(110);
        for ext in ExtensionManager::list().read().await.iter() {
            if let Some(list) = ext.migrations(&self.context).await {
                for m in list {
                    if let Err(e) = m.setup(&self.context).await {
                        tracing::error!("migration setup failed for {}: {}", m.id(), e);
                        continue;
                    }

                    if !m.should_run(&self.context).await {
                        tracing::debug!(
                            "skipping migration {} as should_run returned false",
                            m.id()
                        );
                        continue;
                    }
                    migrations.push(m);
                }
            }
        }

        migrations.reverse();
        migrations
    }
}

impl From<(String, ArgMatches)> for MigrateAction {
    fn from(value: (String, ArgMatches)) -> Self {
        match value.0.to_lowercase().as_str() {
            "up" => MigrateAction::Up,
            "down" => MigrateAction::Down,
            "refresh" => MigrateAction::Refresh,
            "reset" => MigrateAction::Reset,
            "list" => MigrateAction::List,
            _ => panic!("unknown migration action: {}", value.0),
        }
    }
}
