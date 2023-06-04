use super::{AppEntity, APP_TABLE, APP_TABLE_ID_FIELD};
use dirtybase_db::base::{
    field_values::FieldValue, manager::Manager, types::FromColumnAndValue,
    types::IntoColumnAndValue,
};

pub struct AppRepository {
    manager: Manager,
}

impl AppRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub fn manager_mut(&mut self) -> &mut Manager {
        &mut self.manager
    }

    pub async fn find_by_id(&mut self, id: &str) -> Result<AppEntity, anyhow::Error> {
        match self
            .manager_mut()
            .select_from_table(APP_TABLE, |q| {
                q.select_all().eq(APP_TABLE_ID_FIELD, id);
            })
            .fetch_one_as_field_value()
            .await
        {
            Ok(result) => Ok(AppEntity::from_column_value(result)),
            Err(e) => Err(e),
        }
    }

    pub async fn create(
        &mut self,
        record: impl IntoColumnAndValue,
    ) -> Result<AppEntity, anyhow::Error> {
        let kv = record.into_column_value();
        let id: String = FieldValue::from_ref_option_into(kv.get(APP_TABLE_ID_FIELD));

        self.manager.insert(APP_TABLE, kv).await;
        self.find_by_id(&id).await
    }

    pub async fn update(
        &mut self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<AppEntity, anyhow::Error> {
        let column_and_values = record.into_column_value();
        self.manager
            .update(APP_TABLE, column_and_values, |q| {
                q.eq(APP_TABLE_ID_FIELD, id);
            })
            .await;

        self.find_by_id(id).await
    }
}
