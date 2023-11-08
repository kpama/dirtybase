use anyhow::Ok;
use dirtybase_db::base::manager::Manager;
use dirtybase_db_types::{types::IntoColumnAndValue, TableEntityTrait};

use crate::app::{setup_database::setup_migration_table, DirtyBaseApp};

use super::MigrationEntity;

pub struct MigrationRepository {
    manager: Manager,
}

impl MigrationRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub async fn init(&self) {
        setup_migration_table(&self.manager).await;
    }

    pub async fn find_by_name(&self, name: &str) -> Result<MigrationEntity, anyhow::Error> {
        self.manager
            .select_from_table(MigrationEntity::table_name(), |query| {
                query
                    .select_all()
                    .eq(MigrationEntity::col_name_for_name(), name);
            })
            .fetch_one_to()
            .await
    }

    pub async fn get_last_batch(&self) -> Result<Vec<MigrationEntity>, anyhow::Error> {
        Ok(Vec::new())
    }

    pub async fn create(&self, record: MigrationEntity) -> Result<MigrationEntity, anyhow::Error> {
        let name = record.name.as_ref().clone().unwrap().clone();
        let kv = record.into_column_value();
        self.manager.insert(MigrationEntity::table_name(), kv).await;

        self.find_by_name(&name).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for MigrationRepository {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        let app = c.get::<DirtyBaseApp>().unwrap();
        Self::new(app.schema_manger())
    }
}
