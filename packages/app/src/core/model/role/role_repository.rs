use crate::core::App;

use super::RoleEntity;
use dirtybase_contract::db::{base::manager::Manager, types::IntoColumnAndValue};
use dirtybase_db::{field_values::FieldValue, types::UlidField, TableEntityTrait};

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
            .select_from_table(RoleEntity::table_name(), |q| {
                q.eq(RoleEntity::id_column().unwrap(), id);
                if !with_trashed {
                    q.is_null(RoleEntity::deleted_at_column().unwrap());
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
        let id: String = FieldValue::from_ref_option_into(kv.get(RoleEntity::table_name()));

        self.manager.insert(RoleEntity::table_name(), kv).await;
        // ERROR: ID could not be in the hash map!!!
        self.find_by_id(&id, false).await
    }

    pub async fn update(
        &self,
        id: &UlidField,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<RoleEntity>, anyhow::Error> {
        let column_and_values = record.into_column_value();
        self.manager
            .update(RoleEntity::table_name(), column_and_values, |q| {
                q.eq(RoleEntity::id_column().unwrap(), id);
            })
            .await;

        self.find_by_id(id, false).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for RoleRepository {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app = ci.get::<App>().unwrap();

        Self::new(app.schema_manger())
    }
}
