use crate::core::App;

use super::RoleUserEntity;
use dirtybase_contract::db::{
    base::manager::Manager, field_values::FieldValue, types::IntoColumnAndValue,
};
use dirtybase_db::TableEntityTrait;

pub struct RoleUserRepository {
    manager: Manager,
}

impl RoleUserRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub fn manager_mut(&mut self) -> &mut Manager {
        &mut self.manager
    }

    pub async fn find_by_user_and_app_ids(
        &self,
        user_id: &str,
        app_id: &str,
        with_trashed: bool,
    ) -> Result<Option<RoleUserEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(RoleUserEntity::table_name(), |q| {
                q.eq(RoleUserEntity::col_name_for_core_user_id(), user_id)
                    .eq(RoleUserEntity::col_name_for_core_app_role_id(), app_id);
                if !with_trashed {
                    q.is_null(RoleUserEntity::deleted_at_column().unwrap());
                }
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<RoleUserEntity>, anyhow::Error> {
        let kv = record.into_column_value();
        let user_id: String =
            FieldValue::from_ref_option_into(kv.get(RoleUserEntity::col_name_for_core_user_id()));
        let app_id: String = FieldValue::from_ref_option_into(
            kv.get(RoleUserEntity::col_name_for_core_app_role_id()),
        );

        self.manager.insert(RoleUserEntity::table_name(), kv).await;
        self.find_by_user_and_app_ids(&user_id, &app_id, false)
            .await
    }

    pub async fn update(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<RoleUserEntity>, anyhow::Error> {
        let kv = record.into_column_value();

        let user_id: String =
            FieldValue::from_ref_option_into(kv.get(RoleUserEntity::col_name_for_core_user_id()));
        let app_id: String = FieldValue::from_ref_option_into(
            kv.get(RoleUserEntity::col_name_for_core_app_role_id()),
        );

        self.manager
            .update(RoleUserEntity::table_name(), kv, |q| {
                q.eq(RoleUserEntity::col_name_for_core_user_id(), &user_id)
                    .eq(RoleUserEntity::col_name_for_core_app_role_id(), &app_id);
            })
            .await;

        self.find_by_user_and_app_ids(&user_id, &app_id, false)
            .await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for RoleUserRepository {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app = ci.get::<App>().await.unwrap();

        Self::new(app.schema_manger())
    }
}
