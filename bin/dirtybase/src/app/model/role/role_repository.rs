use super::{RoleEntity, ROLE_TABLE, ROLE_TABLE_DELETED_AT_FIELD, ROLE_TABLE_ID_FIELD};
use crate::app::DirtyBaseApp;
use dirtybase_contract::db::{base::manager::Manager, types::IntoColumnAndValue};
use dirtybase_db::field_values::FieldValue;

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

    pub async fn find_by_id_with_trashed(
        &self,
        id: &str,
    ) -> Result<Option<RoleEntity>, anyhow::Error> {
        self.find_by_id(id, true).await
    }

    pub async fn find_by_id(
        &self,
        id: &str,
        with_trashed: bool,
    ) -> Result<Option<RoleEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(ROLE_TABLE, |q| {
                q.select_all().eq(ROLE_TABLE_ID_FIELD, id);
                if !with_trashed {
                    q.is_null(ROLE_TABLE_DELETED_AT_FIELD);
                }
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<RoleEntity>, anyhow::Error> {
        let kv = record.into_column_value();
        let id: String = FieldValue::from_ref_option_into(kv.get(ROLE_TABLE_ID_FIELD));

        self.manager.insert(ROLE_TABLE, kv).await;
        // ERROR: ID could not be in the hash map!!!
        self.find_by_id(&id, false).await
    }

    pub async fn update(
        &self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<RoleEntity>, anyhow::Error> {
        let column_and_values = record.into_column_value();
        self.manager
            .update(ROLE_TABLE, column_and_values, |q| {
                q.eq(ROLE_TABLE_ID_FIELD, id);
            })
            .await;

        self.find_by_id(id, false).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for RoleRepository {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app = ci.get::<DirtyBaseApp>().unwrap();

        Self::new(app.schema_manger())
    }
}
