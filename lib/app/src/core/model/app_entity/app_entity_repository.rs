use super::{AppEntity, APP_TABLE, APP_TABLE_ID_FIELD};
use crate::core::model::{role::RoleEntity, role_user::RoleUserEntity};
use crate::core::App;
use dirtybase_db::TableEntityTrait;
use dirtybase_db::{
    base::manager::Manager,
    field_values::FieldValue,
    types::{IntoColumnAndValue, StructuredColumnAndValue},
};
use dirtybase_user::entity::user::UserEntity;

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

    pub async fn find_all_by_user(
        &self,
        core_user_id: &str,
    ) -> Result<Option<Vec<StructuredColumnAndValue>>, anyhow::Error> {
        self.manager()
            .select_from_table(UserEntity::table_name(), |q| {
                let app_columns = AppEntity::table_column_full_names()
                    .iter()
                    .enumerate()
                    .map(|x| {
                        if x.0 == 0 {
                            format!("distinct {}", x.1)
                        } else {
                            x.1.clone()
                        }
                    })
                    .collect::<Vec<String>>();

                q.select_multiple(&app_columns)
                    .left_join_table::<RoleUserEntity, UserEntity>("core_user_id", "id")
                    .left_join_table::<RoleEntity, RoleUserEntity>("id", "core_app_role_id")
                    .left_join_table::<AppEntity, RoleEntity>("id", "core_app_id")
                    .eq("core_user.id", core_user_id);
            })
            .fetch_all()
            .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<AppEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(APP_TABLE, |q| {
                q.eq(APP_TABLE_ID_FIELD, id);
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<AppEntity>, anyhow::Error> {
        let kv = record.into_column_value();
        let id: String = FieldValue::from_ref_option_into(kv.get(APP_TABLE_ID_FIELD));

        self.manager.insert(APP_TABLE, kv).await;
        self.find_by_id(&id).await
    }

    pub async fn update(
        &self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<AppEntity>, anyhow::Error> {
        let column_and_values = record.into_column_value();
        self.manager
            .update(APP_TABLE, column_and_values, |q| {
                q.eq(APP_TABLE_ID_FIELD, id);
            })
            .await;

        self.find_by_id(id).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for AppRepository {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app = ci.get::<App>().unwrap();
        Self::new(app.schema_manger())
    }
}
