use super::{SysAdminEntity, SYS_ADMIN_TABLE, SYS_ADMIN_TABLE_USER_ID_FIELD};
use dirtybase_contract::db::{
    base::manager::Manager, field_values::FieldValue, types::IntoColumnAndValue,
};

pub struct SysAdminRepository {
    manager: Manager,
}

impl SysAdminRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub fn manager_mut(&mut self) -> &mut Manager {
        &mut self.manager
    }

    pub async fn find_by_user_id(&self, id: &str) -> Result<Option<SysAdminEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(SYS_ADMIN_TABLE, |q| {
                q.eq(SYS_ADMIN_TABLE_USER_ID_FIELD, id);
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<SysAdminEntity>, anyhow::Error> {
        let kv = record.into_column_value();
        let user_id: String =
            FieldValue::from_ref_option_into(kv.get(SYS_ADMIN_TABLE_USER_ID_FIELD));

        self.manager.insert(SYS_ADMIN_TABLE, kv).await;
        self.find_by_user_id(&user_id).await
    }

    pub async fn delete(&self, user_id: &str) -> Result<bool, anyhow::Error> {
        self.manager
            .delete(SYS_ADMIN_TABLE, |q| {
                q.eq(SYS_ADMIN_TABLE_USER_ID_FIELD, user_id);
            })
            .await;
        Ok(true)
    }
}
