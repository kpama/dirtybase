use std::collections::HashMap;

use dirtybase_db::base::manager::Manager;
use dirtybase_db::TableEntityTrait;

use crate::core::setup_database::setup_migration_table;
use crate::core::App;

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

    pub async fn exist(&self, name: &str) -> bool {
        match self.find_by_name(name).await {
            Ok(Some(_)) => true,
            _ => false,
        }
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<MigrationEntity>, anyhow::Error> {
        self.manager
            .select_from_table(MigrationEntity::table_name(), |query| {
                query
                    .select_all()
                    .eq(MigrationEntity::col_name_for_name(), name);
            })
            .fetch_one_to()
            .await
    }

    pub async fn get_last_batch(&self) -> Result<HashMap<String, MigrationEntity>, anyhow::Error> {
        Ok(HashMap::new())
    }

    pub async fn create(
        &self,
        name: &str,
        batch: i64,
    ) -> Result<Option<MigrationEntity>, anyhow::Error> {
        let mut kv = HashMap::new();

        kv.insert("name".to_owned(), name.to_string().into());
        kv.insert("batch".to_owned(), batch.to_string().into());

        self.manager.insert(MigrationEntity::table_name(), kv).await;

        self.find_by_name(name).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for MigrationRepository {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        let app = c.get::<App>().unwrap();
        Self::new(app.schema_manger())
    }
}
