#![allow(dead_code)]

use crate::core::App;

use super::CompanyEntity;
use dirtybase_db::{
    base::manager::Manager, field_values::FieldValue, types::IntoColumnAndValue, TableModel,
};

pub struct CompanyRepository {
    manager: Manager,
}

impl CompanyRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub async fn find_by_internal_id(
        &self,
        id: u64,
    ) -> Result<Option<CompanyEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(CompanyEntity::table_name(), |q| {
                q.eq(CompanyEntity::col_name_for_internal_id(), id)
                    .and_is_null(CompanyEntity::col_name_for_deleted_at());
            })
            .fetch_one_to()
            .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<CompanyEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(CompanyEntity::table_name(), |q| {
                q.eq(CompanyEntity::col_name_for_id(), id)
                    .and_is_null(CompanyEntity::col_name_for_deleted_at());
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<CompanyEntity>, anyhow::Error> {
        let kv = record.into_column_value();
        let id: String = FieldValue::from_ref_option_into(kv.get(CompanyEntity::col_name_for_id()));

        self.manager.insert(CompanyEntity::table_name(), kv).await;
        self.find_by_id(&id).await
    }

    pub async fn update(
        &self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<CompanyEntity>, anyhow::Error> {
        let column_and_values = record.into_column_value();
        self.manager
            .update(CompanyEntity::table_name(), column_and_values, |q| {
                q.eq(CompanyEntity::col_name_for_id(), id);
            })
            .await;

        self.find_by_id(id).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for CompanyRepository {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        let app = c.get::<App>().await.unwrap();
        Self::new(app.schema_manger().await)
    }
}
