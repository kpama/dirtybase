use super::{
    dirtybase_user_entity::DirtybaseUserEntity, in_dtos::UserLoginPayload, out_dtos::LoggedInUser,
    DIRTYBASE_USER_TABLE, DIRTYBASE_USER_TABLE_CORE_USER_FIELD,
};
use crate::app::DirtyBase;
use dirtybase_db::{
    base::{manager::Manager, query::QueryBuilder},
    dirtybase_db_types::{field_values::FieldValue, types::IntoColumnAndValue},
    entity::user::{
        UserEntity, UserRepository, USER_TABLE, USER_TABLE_EMAIL_FIELD, USER_TABLE_USERNAME_FIELD,
    },
};

pub struct DirtybaseUserRepository {
    manager: Manager,
    user_repo: UserRepository,
}

impl DirtybaseUserRepository {
    pub fn new(manager: Manager, user_repo: UserRepository) -> Self {
        Self { manager, user_repo }
    }

    pub fn user_repo(&self) -> &UserRepository {
        &self.user_repo
    }

    pub async fn find_by_username_or_email(
        &self,
        username: &str,
        email: &str,
        without_trash: bool,
    ) -> Result<UserEntity, anyhow::Error> {
        if !username.is_empty() || !email.is_empty() {
            self.user_repo()
                .manager()
                .select_from_table(USER_TABLE, |q| {
                    q.select_all();
                    if without_trash {
                        q.without_trash();
                    }
                    if !email.is_empty() {
                        q.eq(USER_TABLE_EMAIL_FIELD, email);
                    } else {
                        q.eq(USER_TABLE_USERNAME_FIELD, username);
                    }
                })
                .fetch_one_to()
                .await
        } else {
            Err(anyhow::anyhow!("Both username and email values are empty"))
        }
    }

    pub async fn find_user_for_login(
        &self,
        _payload: &UserLoginPayload,
    ) -> Result<LoggedInUser, anyhow::Error> {
        //    let result=  self.manager.select_from_table(USER_TABLE, |q|{
        //         q.select("core_user.id")
        //             .select("core_user.username")
        //             .select("core_user.status")
        //             .select("core_app_role.core_app_id")
        //             .left_join_and_select("core_app_role", "core_app_role" , "=" , right_table, select_columns)

        //    }).fetch_one().await;
        //    dbg!(result);
        Ok(LoggedInUser::default())
    }

    pub async fn fin_by_core_user_id(
        &self,
        core_user_id: &str,
    ) -> Result<DirtybaseUserEntity, anyhow::Error> {
        self.manager
            .select_from_table(DIRTYBASE_USER_TABLE, |q| {
                q.select_all();
                q.eq(DIRTYBASE_USER_TABLE_CORE_USER_FIELD, core_user_id);
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<DirtybaseUserEntity, anyhow::Error> {
        let column_and_values = record.into_column_value();
        let core_user_id: String = FieldValue::from_ref_option_into(
            column_and_values.get(DIRTYBASE_USER_TABLE_CORE_USER_FIELD),
        );
        self.manager
            .insert(DIRTYBASE_USER_TABLE, column_and_values)
            .await;

        self.fin_by_core_user_id(&core_user_id).await
    }

    fn build_query(query: &mut QueryBuilder) {
        // TODO: ADD joins here
    }
}

#[busybody::async_trait]
impl busybody::Injectable for DirtybaseUserRepository {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app = ci.get::<DirtyBase>().unwrap();

        Self::new(
            app.schema_manger(),
            UserRepository::new(app.schema_manger()),
        )
    }
}
