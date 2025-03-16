use dirtybase_contract::db::base::manager::Manager;
use std::collections::{BTreeMap, HashMap};

use super::{BATCH_COLUMN, MigrationEntity, NAME_COLUMN, TABLE_NAME};

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
            .await;
    }

    pub async fn exist(&self, name: &str) -> bool {
        matches!(self.find_by_name(name).await, Ok(Some(_)))
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<MigrationEntity>, anyhow::Error> {
        self.manager
            .select_from_table(TABLE_NAME, |query| {
                query.eq(NAME_COLUMN, name);
            })
            .fetch_one_to()
            .await
    }

    pub async fn get_last_batch(&self) -> BTreeMap<String, MigrationEntity> {
        if let Ok(Some(last)) = self
            .manager
            .select_from_table(TABLE_NAME, |q| {
                q.desc(BATCH_COLUMN);
            })
            .fetch_one_to::<MigrationEntity>()
            .await
        {
            tracing::debug!("last migration {:?}", &last);

            if let Ok(Some(collection)) = self
                .manager
                .select_from_table(TABLE_NAME, |q| {
                    q.eq(BATCH_COLUMN, last.batch).desc("created_at");
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
        _ = self
            .manager()
            .delete(TABLE_NAME, |q| {
                q.eq(BATCH_COLUMN, batch);
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
        kv.insert("batch".to_owned(), batch.into());

        _ = self.manager.insert(TABLE_NAME, kv).await;

        self.find_by_name(name).await
    }
}
