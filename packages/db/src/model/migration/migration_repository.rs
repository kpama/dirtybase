use dirtybase_contract::db_contract::{base::manager::Manager, types::StringField};
use dirtybase_helper::time::current_datetime;
use std::collections::{BTreeMap, HashMap};

use super::{BATCH_COLUMN, CREATED_AT_COLUMN, MigrationEntity, NAME_COLUMN, TABLE_NAME};

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

    pub async fn init(&self) -> Result<(), anyhow::Error> {
        self.manager
            .create_table_schema(TABLE_NAME, |table| {
                // id
                table.id(Some("id"));

                // migration name
                table.text(NAME_COLUMN);

                // batch
                table.integer(BATCH_COLUMN);

                // created at
                table.created_at();
            })
            .await
    }

    pub async fn exist(&self, name: &str) -> bool {
        matches!(self.find_by_name(name).await, Ok(Some(_)))
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<MigrationEntity>, anyhow::Error> {
        self.manager
            .select_from_table(TABLE_NAME, |query| {
                query.is_eq(NAME_COLUMN, name);
            })
            .fetch_one_to()
            .await
    }

    pub async fn get_last_batch(&self) -> BTreeMap<StringField, MigrationEntity> {
        if let Ok(Some(last)) = self
            .manager
            .select_from_table(TABLE_NAME, |q| {
                q.desc(BATCH_COLUMN);
            })
            .fetch_one_to::<MigrationEntity>()
            .await
        {
            if let Ok(Some(collection)) = self
                .manager
                .select_from_table(TABLE_NAME, |q| {
                    q.is_eq(BATCH_COLUMN, last.batch).desc("created_at");
                })
                .fetch_all_to::<MigrationEntity>()
                .await
            {
                return collection
                    .into_iter()
                    .map(|m| (m.name.clone(), m))
                    .collect::<BTreeMap<StringField, MigrationEntity>>();
            }
        }

        BTreeMap::new()
    }

    pub async fn delete_batch(&self, batch: i64) {
        _ = self
            .manager()
            .delete(TABLE_NAME, |q| {
                q.is_eq(BATCH_COLUMN, batch);
            })
            .await;
    }

    pub async fn create(
        &self,
        name: &str,
        batch: i64,
    ) -> Result<Option<MigrationEntity>, anyhow::Error> {
        let mut kv = HashMap::new();

        kv.insert(NAME_COLUMN.to_string(), name.to_string().into());
        kv.insert(BATCH_COLUMN.to_string(), batch.into());
        kv.insert(CREATED_AT_COLUMN.to_string(), current_datetime().into());

        _ = self.manager.insert(TABLE_NAME, kv).await;

        self.find_by_name(name).await
    }
}
