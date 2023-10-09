use std::collections::HashMap;

use dirtybase_db::base::manager::Manager;
use dirtybase_db_types::{field_values::FieldValue, TableEntityTrait};
use dirtybase_helper::time::now;

use super::{CacheDbPivotEntity, CacheDbStoreEntity, CacheDbTagStoreEntity};

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
        keys: &[String],
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
                            now().timestamp(),
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

    pub async fn update_many(
        &self,
        kv: &HashMap<String, String>,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        for entry in kv {
            self.create(&entry.0, &entry.1, expiration, tags).await;
        }
        true
    }

    pub async fn update(
        &self,
        key: &str,
        data: &str,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
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
            self.tag_key(tags, key).await;
            return true;
        }
        return false;
    }

    pub async fn create(
        &self,
        key: &str,
        data: &str,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let result = self.update(key, data, expiration, tags).await;

        if !result {
            let payload = self.build_payload(key, data, expiration);
            self.manager
                .insert(CacheDbStoreEntity::table_name(), payload)
                .await;
            self.tag_key(tags, key).await;
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

    pub async fn delete_all(&self, tags: Option<&[String]>) -> bool {
        if tags.is_some() {
            self.manager
                .delete(CacheDbStoreEntity::table_name(), |query| {
                    query
                        .left_join_table::<CacheDbPivotEntity, CacheDbStoreEntity>(
                            CacheDbPivotEntity::col_name_for_core_cache_key(),
                            CacheDbStoreEntity::col_name_for_key(),
                        )
                        .is_in(
                            CacheDbPivotEntity::prefix_with_tbl(
                                CacheDbPivotEntity::col_name_for_core_cache_tags_id(),
                            ),
                            tags,
                        );
                })
                .await;

            self.delete_tags(tags).await;

            return true;
        }

        return false;
    }

    pub async fn delete_expired(&self) {
        self.manager
            .delete(CacheDbStoreEntity::table_name(), |query| {
                query.gt_or_eq(
                    CacheDbStoreEntity::col_name_for_expiration(),
                    now().timestamp(),
                );
            })
            .await;
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

    async fn insert_tags(&self, tags: Option<&[String]>) -> bool {
        if tags.is_some() {
            let tags = tags.unwrap();
            let rows: Vec<HashMap<String, FieldValue>> = tags
                .iter()
                .map(|t| {
                    HashMap::from([(
                        CacheDbTagStoreEntity::col_name_for_tag().to_string(),
                        FieldValue::from(t),
                    )])
                })
                .collect();

            self.manager
                .soft_insert_multi(CacheDbTagStoreEntity::table_name(), rows)
                .await;
            return true;
        }

        false
    }

    async fn delete_tags(&self, tags: Option<&[String]>) {
        self.manager
            .delete(CacheDbTagStoreEntity::table_name(), |query| {
                query.is_in(CacheDbTagStoreEntity::col_name_for_tag(), tags);
            })
            .await;
    }

    async fn tag_key(&self, tags: Option<&[String]>, cache_key: &str) -> bool {
        self.insert_tags(tags).await;
        if tags.is_some() {
            let rows = tags
                .unwrap()
                .iter()
                .map(|a_tag| {
                    HashMap::from([
                        (
                            // Cache entry key
                            CacheDbPivotEntity::col_name_for_core_cache_key().to_string(),
                            FieldValue::String(cache_key.to_string()),
                        ),
                        (
                            // Tag's ID
                            CacheDbPivotEntity::col_name_for_core_cache_tags_id().to_string(),
                            FieldValue::String(a_tag.clone()),
                        ),
                    ])
                })
                .collect::<Vec<HashMap<String, FieldValue>>>();

            self.manager
                .soft_insert_multi(CacheDbPivotEntity::table_name(), rows)
                .await; // TODO: LOG Errors
            return true;
        }

        false
    }
}

#[busybody::async_trait]
impl busybody::Injectable for CacheDbStoreRepository {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        let pool_manager = c.get::<dirtybase_db::ConnectionPoolManager>().unwrap();
        Self::new(pool_manager.default_schema_manager().unwrap())
    }
}
