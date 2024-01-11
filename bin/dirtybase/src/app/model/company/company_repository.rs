#![allow(dead_code)]

use super::{
    CompanyEntity, COMPANY_TABLE, COMPANY_TABLE_DELETED_AT_FIELD, COMPANY_TABLE_ID_FIELD,
    COMPANY_TABLE_INTERNAL_ID_FIELD,
};
use crate::app::DirtyBaseApp;
use dirtybase_db::{base::manager::Manager, field_values::FieldValue, types::IntoColumnAndValue};

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
            .select_from_table(COMPANY_TABLE, |q| {
                q.select_all()
                    .eq(COMPANY_TABLE_INTERNAL_ID_FIELD, id)
                    .and_is_null(COMPANY_TABLE_DELETED_AT_FIELD);
            })
            .fetch_one_to()
            .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<CompanyEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(COMPANY_TABLE, |q| {
                q.select_all()
                    .eq(COMPANY_TABLE_ID_FIELD, id)
                    .and_is_null(COMPANY_TABLE_DELETED_AT_FIELD);
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<CompanyEntity>, anyhow::Error> {
        let kv = record.into_column_value();
        let id: String = FieldValue::from_ref_option_into(kv.get(COMPANY_TABLE_ID_FIELD));

        self.manager.insert(COMPANY_TABLE, kv).await;
        self.find_by_id(&id).await
    }

    pub async fn update(
        &self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<CompanyEntity>, anyhow::Error> {
        let column_and_values = record.into_column_value();
        self.manager
            .update(COMPANY_TABLE, column_and_values, |q| {
                q.eq(COMPANY_TABLE_ID_FIELD, id);
            })
            .await;

        self.find_by_id(id).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for CompanyRepository {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        let app = c.get::<DirtyBaseApp>().unwrap();
        Self::new(app.schema_manger())
    }
}
