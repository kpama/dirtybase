use std::collections::HashMap;

use crate::app::DirtyBase;
use dirtybase_db::base::manager::Manager;
use dirtybase_db_types::{field_values::FieldValue, TableEntityTrait};

use super::CacheDbStoreEntity;

pub struct CacheDbStoreRepository {
    manager: Manager,
}

impl CacheDbStoreRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub async fn get(&self, key: &str, with_trashed: bool) -> Option<CacheDbStoreEntity> {
        match self
            .manager
            .select_from_table(CacheDbStoreEntity::table_name(), |query| {
                query.select_all();
                if !with_trashed {
                    // TODO: Add time condition
                }

                query.eq(
                    CacheDbStoreEntity::prefix_with_tbl(CacheDbStoreEntity::col_name_for_key()),
                    key,
                );
            })
            .fetch_one_to()
            .await
        {
            Ok(result) => Some(result),
            _ => None,
        }
    }

    pub async fn get_many(
        &self,
        keys: &[&str],
        with_trashed: bool,
    ) -> Option<Vec<CacheDbStoreEntity>> {
        match self
            .manager
            .select_from_table(CacheDbStoreEntity::table_name(), |query| {
                query.select_all();

                query.is_in(
                    CacheDbStoreEntity::prefix_with_tbl(CacheDbStoreEntity::col_name_for_key()),
                    keys.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
                );

                if !with_trashed {
                    query
                        .is_null(CacheDbStoreEntity::prefix_with_tbl(
                            CacheDbStoreEntity::col_name_for_expiration(),
                        ))
                        .or_le_or_eq(
                            CacheDbStoreEntity::col_name_for_expiration(),
                            chrono::Utc::now().timestamp(),
                        );
                }
            })
            .fetch_all_to::<CacheDbStoreEntity>()
            .await
        {
            Ok(list) => Some(list),
            _ => None,
        }
    }

    pub async fn update_many(&self, kv: &HashMap<String, String>, expiration: Option<i64>) -> bool {
        for entry in kv {
            self.insert(&entry.0, &entry.1, expiration).await;
        }
        true
    }

    pub async fn update(&self, key: &str, data: &str, expiration: Option<i64>) -> bool {
        let result = self.get(key, true).await;
        if result.is_some() {
            let payload = self.build_payload(key, data, expiration);
            self.manager
                .update(CacheDbStoreEntity::table_name(), payload, |query| {
                    query.eq(
                        CacheDbStoreEntity::prefix_with_tbl(CacheDbStoreEntity::col_name_for_key()),
                        key,
                    );
                })
                .await;
            return true;
        }
        return false;
    }

    pub async fn insert(&self, key: &str, data: &str, expiration: Option<i64>) -> bool {
        let result = self.update(key, data, expiration).await;

        if !result {
            let payload = self.build_payload(key, data, expiration);
            self.manager
                .insert(CacheDbStoreEntity::table_name(), payload)
                .await;
            return true;
        }

        return false;
    }

    pub async fn delete(&self, key: &str) -> bool {
        self.manager
            .delete(CacheDbStoreEntity::table_name(), |query| {
                query.eq(CacheDbStoreEntity::col_name_for_key(), key);
            })
            .await;
        return true;
    }

    pub async fn delete_all(&self) -> bool {
        self.manager
            .delete(CacheDbStoreEntity::table_name(), |_| {
                // No filter
            })
            .await;
        return true;
    }

    fn build_payload(
        &self,
        key: &str,
        data: &str,
        expiration: Option<i64>,
    ) -> HashMap<String, FieldValue> {
        let mut payload = HashMap::new();
        payload.insert(CacheDbStoreEntity::col_name_for_key().into(), key.into());

        if !data.is_empty() {
            payload.insert(
                CacheDbStoreEntity::col_name_for_content().to_string(),
                data.into(),
            );
        }

        if expiration.is_some() {
            payload.insert(
                CacheDbStoreEntity::col_name_for_expiration().to_string(),
                expiration.unwrap().into(),
            );
        }

        payload
    }
}

#[busybody::async_trait]
impl busybody::Injectable for CacheDbStoreRepository {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        let app = c.get::<DirtyBase>().unwrap();
        Self::new(app.schema_manger())
    }
}
