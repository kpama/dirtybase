use super::{RoleEntity, ROLE_TABLE, ROLE_TABLE_DELETED_AT_FIELD, ROLE_TABLE_ID_FIELD};
use dirtybase_db::base::{
    field_values::FieldValue, manager::Manager, types::FromColumnAndValue,
    types::IntoColumnAndValue,
};

pub struct RoleRepository {
    manager: Manager,
}

impl RoleRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub fn manager_mut(&mut self) -> &mut Manager {
        &mut self.manager
    }

    pub async fn find_by_id_with_trashed(&mut self, id: &str) -> Result<RoleEntity, anyhow::Error> {
        self.find_by_id(id, true).await
    }

    pub async fn find_by_id(
        &mut self,
        id: &str,
        with_trashed: bool,
    ) -> Result<RoleEntity, anyhow::Error> {
        match self
            .manager_mut()
            .select_from_table(ROLE_TABLE, |q| {
                q.select_all().eq(ROLE_TABLE_ID_FIELD, id);
                if !with_trashed {
                    q.is_null(ROLE_TABLE_DELETED_AT_FIELD);
                }
            })
            .fetch_one()
            .await
        {
            Ok(result) => Ok(RoleEntity::from_column_value(result)),
            Err(e) => Err(e),
        }
    }

    pub async fn create(
        &mut self,
        record: impl IntoColumnAndValue,
    ) -> Result<RoleEntity, anyhow::Error> {
        let kv = record.into_column_value();
        let id: String = FieldValue::from_ref_option_into(kv.get(ROLE_TABLE_ID_FIELD));

        self.manager.insert(ROLE_TABLE, kv).await;
        self.find_by_id(&id, false).await
    }

    pub async fn update(
        &mut self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<RoleEntity, anyhow::Error> {
        let column_and_values = record.into_column_value();
        self.manager
            .update(ROLE_TABLE, column_and_values, |q| {
                q.eq(ROLE_TABLE_ID_FIELD, id);
            })
            .await;

        self.find_by_id(id, false).await
    }
}
