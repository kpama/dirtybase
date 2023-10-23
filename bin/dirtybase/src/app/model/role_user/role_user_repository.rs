use super::{
    RoleUserEntity, ROLE_USER_TABLE, ROLE_USER_TABLE_CORE_APP_ROLE_ID_FIELD,
    ROLE_USER_TABLE_CORE_USER_ID_FIELD, ROLE_USER_TABLE_DELETED_AT_FIELD,
};
use crate::app::DirtyBaseApp;
use dirtybase_db::{
    base::manager::Manager,
    dirtybase_db_types::{field_values::FieldValue, types::IntoColumnAndValue},
};

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
    ) -> Result<RoleUserEntity, anyhow::Error> {
        self.manager()
            .select_from_table(ROLE_USER_TABLE, |q| {
                q.select_all()
                    .eq(ROLE_USER_TABLE_CORE_USER_ID_FIELD, user_id)
                    .eq(ROLE_USER_TABLE_CORE_APP_ROLE_ID_FIELD, app_id);
                if !with_trashed {
                    q.is_null(ROLE_USER_TABLE_DELETED_AT_FIELD);
                }
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<RoleUserEntity, anyhow::Error> {
        let kv = record.into_column_value();
        let user_id: String =
            FieldValue::from_ref_option_into(kv.get(ROLE_USER_TABLE_CORE_USER_ID_FIELD));
        let app_id: String =
            FieldValue::from_ref_option_into(kv.get(ROLE_USER_TABLE_CORE_APP_ROLE_ID_FIELD));

        self.manager.insert(ROLE_USER_TABLE, kv).await;
        self.find_by_user_and_app_ids(&user_id, &app_id, false)
            .await
    }

    pub async fn update(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<RoleUserEntity, anyhow::Error> {
        let kv = record.into_column_value();

        let user_id: String =
            FieldValue::from_ref_option_into(kv.get(ROLE_USER_TABLE_CORE_USER_ID_FIELD));
        let app_id: String =
            FieldValue::from_ref_option_into(kv.get(ROLE_USER_TABLE_CORE_APP_ROLE_ID_FIELD));

        self.manager
            .update(ROLE_USER_TABLE, kv, |q| {
                q.eq(ROLE_USER_TABLE_CORE_USER_ID_FIELD, &user_id)
                    .eq(ROLE_USER_TABLE_CORE_APP_ROLE_ID_FIELD, &app_id);
            })
            .await;

        self.find_by_user_and_app_ids(&user_id, &app_id, false)
            .await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for RoleUserRepository {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app = ci.get::<DirtyBaseApp>().unwrap();

        Self::new(app.schema_manger())
    }
}
