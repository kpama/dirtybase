use std::collections::{BTreeMap, HashMap};

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
        // if let Ok(Some(_)) = self.find_by_name(name).await {
        //     true
        // } else {
        //     false
        // }
        matches!(self.find_by_name(name).await, Ok(Some(_)))
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<MigrationEntity>, anyhow::Error> {
        self.manager
            .select_from_table(MigrationEntity::table_name(), |query| {
                query.eq(MigrationEntity::col_name_for_name(), name);
            })
            .fetch_one_to()
            .await
    }

    pub async fn get_last_batch(&self) -> BTreeMap<String, MigrationEntity> {
        if let Ok(Some(last)) = self
            .manager
            .select_from_table(MigrationEntity::table_name(), |q| {
                q.desc(MigrationEntity::col_name_for_batch());
            })
            .fetch_one_to::<MigrationEntity>()
            .await
        {
            dbg!("{}", &last);

            if let Ok(Some(collection)) = self
                .manager
                .select_from_table(MigrationEntity::table_name(), |q| {
                    q.eq(MigrationEntity::col_name_for_batch(), last.batch)
                        .desc(MigrationEntity::col_name_for_created_at());
                })
                .fetch_all_to::<MigrationEntity>()
                .await
            {
                return collection
                    .into_iter()
                    .map(|m| (m.name.clone(), m))
                    .collect::<BTreeMap<String, MigrationEntity>>();
            }
        }

        BTreeMap::new()
    }

    pub async fn delete_batch(&self, batch: i64) {
        self.manager()
            .delete(MigrationEntity::table_name(), |q| {
                q.eq(MigrationEntity::col_name_for_batch(), batch);
            })
            .await;
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
