#![allow(dead_code)]

use super::{
    CompanyEntity, COMPANY_TABLE, COMPANY_TABLE_DELETED_AT_FIELD, COMPANY_TABLE_ID_FIELD,
    COMPANY_TABLE_INTERNAL_ID_FIELD,
};
use dirtybase_db::base::{
    field_values::FieldValue,
    manager::Manager,
    types::{FromColumnAndValue, IntoColumnAndValue},
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

    pub fn manager_mut(&mut self) -> &mut Manager {
        &mut self.manager
    }

    pub async fn find_by_internal_id(&mut self, id: u64) -> Result<CompanyEntity, anyhow::Error> {
        match self
            .manager_mut()
            .select_from_table(COMPANY_TABLE, |q| {
                q.select_all()
                    .eq(COMPANY_TABLE_INTERNAL_ID_FIELD, id)
                    .and_is_null(COMPANY_TABLE_DELETED_AT_FIELD);
            })
            .fetch_one_as_field_value()
            .await
        {
            Ok(result) => Ok(CompanyEntity::from_column_value(result)),
            Err(e) => Err(e),
        }
    }

    pub async fn find_by_id(&mut self, id: &str) -> Result<CompanyEntity, anyhow::Error> {
        match self
            .manager_mut()
            .select_from_table(COMPANY_TABLE, |q| {
                q.select_all()
                    .eq(COMPANY_TABLE_ID_FIELD, id)
                    .and_is_null(COMPANY_TABLE_DELETED_AT_FIELD);
            })
            .fetch_one_as_field_value()
            .await
        {
            Ok(result) => Ok(CompanyEntity::from_column_value(result)),
            Err(e) => Err(e),
        }
    }

    pub async fn create(
        &mut self,
        record: impl IntoColumnAndValue,
    ) -> Result<CompanyEntity, anyhow::Error> {
        let kv = record.into_column_value();
        let id: String = FieldValue::from_ref_option_into(kv.get(COMPANY_TABLE_ID_FIELD));

        self.manager.insert(COMPANY_TABLE, kv).await;
        self.find_by_id(&id).await
    }

    pub async fn update(
        &mut self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<CompanyEntity, anyhow::Error> {
        let column_and_values = record.into_column_value();
        self.manager
            .update(COMPANY_TABLE, column_and_values, |q| {
                q.eq(COMPANY_TABLE_ID_FIELD, id);
            })
            .await;

        self.find_by_id(id).await
    }
}
