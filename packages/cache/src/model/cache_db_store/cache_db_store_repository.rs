use std::collections::HashMap;

use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::base::query::QueryBuilder;
use dirtybase_contract::db_contract::{TableModel, field_values::FieldValue};
use dirtybase_helper::time::now;

use crate::CacheEntry;

use super::{CacheDbPivotEntity, CacheDbStoreEntity, CacheDbTagStoreEntity};

#[derive(Debug, Clone)]
pub struct CacheDbStoreRepository {
    manager: Manager,
}

impl CacheDbStoreRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub async fn get(&self, key: &str, with_trashed: bool) -> Option<CacheDbStoreEntity> {
        let find_by_key = |query: &mut QueryBuilder| {
            if !with_trashed {
                // TODO: Add time condition
            }

            query.is_eq(
                CacheDbStoreEntity::prefix_with_tbl(CacheEntry::col_name_for_key()),
                key,
            );
        };

        self.manager
            .select_from_table(CacheDbStoreEntity::table_name(), find_by_key)
            .fetch_one_to()
            .await
            .unwrap_or_default()
    }

    pub async fn get_many(
        &self,
        keys: &[String],
        with_trashed: bool,
    ) -> Option<Vec<CacheDbStoreEntity>> {
        let query_by_keys = |query: &mut QueryBuilder| {
            query.is_in(
                CacheDbStoreEntity::prefix_with_tbl(CacheEntry::col_name_for_key()),
                keys.iter().map(|s| s.to_string()).collect::<Vec<String>>(),
            );

            if !with_trashed {
                query
                    .is_null(CacheDbStoreEntity::prefix_with_tbl(
                        CacheEntry::col_name_for_expiration(),
                    ))
                    .or_le_or_eq(CacheEntry::col_name_for_expiration(), now().timestamp());
            }
        };

        self.manager
            .select_from_table(CacheDbStoreEntity::table_name(), query_by_keys)
            .fetch_all_to::<CacheDbStoreEntity>()
            .await
            .unwrap_or_default()
    }

    pub async fn update_many(
        &self,
        kv: HashMap<String, serde_json::Value>,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        for entry in kv {
            self.create(entry.0, entry.1, expiration, tags).await;
        }
        true
    }

    pub async fn update(
        &self,
        key: String,
        data: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let payload = self.build_payload(key.clone(), data, expiration);
        if self
            .manager
            .upsert(
                CacheDbStoreEntity::table_name(),
                payload,
                &[
                    CacheEntry::col_name_for_value(),
                    CacheEntry::col_name_for_expiration(),
                ],
                &[CacheEntry::col_name_for_key()],
            )
            .await
            .is_err()
        {
            return false;
        }
        self.tag_key(tags, &key).await
    }

    pub async fn create(
        &self,
        key: String,
        data: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let payload = self.build_payload(key.clone(), data, expiration);
        if self
            .manager
            .upsert(
                CacheDbStoreEntity::table_name(),
                payload,
                &[
                    CacheEntry::col_name_for_value(),
                    CacheEntry::col_name_for_expiration(),
                ],
                &[CacheEntry::col_name_for_key()],
            )
            .await
            .is_err()
        {
            return false;
        }
        self.tag_key(tags, &key).await
    }

    pub async fn delete(&self, key: &str) -> bool {
        self.manager
            .delete(CacheDbStoreEntity::table_name(), |query| {
                query.is_eq(CacheEntry::col_name_for_key(), key);
            })
            .await
            .is_ok()
    }

    pub async fn delete_all(&self, tags: Option<&[String]>) -> bool {
        if tags.is_some() {
            if self
                .manager
                .delete(CacheDbStoreEntity::table_name(), |query| {
                    query
                        .left_join_table::<CacheDbPivotEntity, CacheDbStoreEntity>(
                            CacheDbPivotEntity::col_name_for_core_cache_key(),
                            CacheEntry::col_name_for_key(),
                        )
                        .is_in(
                            CacheDbPivotEntity::prefix_with_tbl(
                                CacheDbPivotEntity::col_name_for_core_cache_tags_id(),
                            ),
                            tags,
                        );
                })
                .await
                .is_err()
            {
                return false;
            }

            self.delete_tags(tags).await;

            return true;
        }

        false
    }

    pub async fn delete_expired(&self) {
        _ = self
            .manager
            .delete(CacheDbStoreEntity::table_name(), |query| {
                query.gt_or_eq(CacheEntry::col_name_for_expiration(), now().timestamp());
            })
            .await;
    }

    fn build_payload(
        &self,
        key: String,
        data: serde_json::Value,
        expiration: Option<i64>,
    ) -> CacheDbStoreEntity {
        CacheEntry::new(key, data, expiration).into()
    }

    async fn insert_tags(&self, tags: Option<&[String]>) -> bool {
        if let Some(tags) = tags {
            let rows: Vec<HashMap<String, FieldValue>> = tags
                .iter()
                .map(|t| {
                    HashMap::from([(
                        CacheDbTagStoreEntity::col_name_for_tag().to_string(),
                        FieldValue::from(t),
                    )])
                })
                .collect();

            return self
                .manager
                .soft_insert_multi(CacheDbTagStoreEntity::table_name(), rows)
                .await
                .is_ok();
        }

        false
    }

    async fn delete_tags(&self, tags: Option<&[String]>) {
        _ = self
            .manager
            .delete(CacheDbTagStoreEntity::table_name(), |query| {
                query.is_in(CacheDbTagStoreEntity::col_name_for_tag(), tags);
            })
            .await;
    }

    async fn tag_key(&self, tags: Option<&[String]>, cache_key: &str) -> bool {
        self.insert_tags(tags).await;
        if let Some(tags) = tags {
            let rows = tags
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

            return self
                .manager
                .soft_insert_multi(CacheDbPivotEntity::table_name(), rows)
                .await
                .is_ok();
        }

        false
    }
}
