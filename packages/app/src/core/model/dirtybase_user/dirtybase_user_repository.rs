use super::{
    dirtybase_user_entity::DirtybaseUserEntity,
    dtos::{out_logged_in_user_dto::LoggedInUser, out_user_app::UserAppDto},
};
use crate::core::{
    model::{
        app_entity::AppEntity, company::CompanyEntity, role::RoleEntity, role_user::RoleUserEntity,
        sys_admin::SysAdminEntity,
    },
    App,
};
use dirtybase_contract::db::{
    base::{manager::Manager, query::QueryBuilder},
    types::{FromColumnAndValue, IntoColumnAndValue, StructuredColumnAndValue},
};

use dirtybase_db::{field_values::FieldValue, TableEntityTrait};
use dirtybase_user::entity::user::UserEntity;
use std::collections::HashMap;

const USER_JOIN_PREFIX: &str = "user";
const SYS_ADMIN_JOIN_PREFIX: &str = "sys_admin_entity";
const APPS_JOIN_PREFIX: &str = "apps";
const ROLES_JOIN_PREFIX: &str = "apps.roles";
const COMPANY_JOIN_PREFIX: &str = "apps.company";

pub struct DirtybaseUserRepository {
    manager: Manager,
}

impl DirtybaseUserRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub async fn find_by_username_or_email(
        &self,
        username: &str,
        email: &str,
        without_trash: bool,
    ) -> Result<DirtybaseUserEntity, anyhow::Error> {
        if !username.is_empty() || !email.is_empty() {
            self.manager
                .select_from_table(DirtybaseUserEntity::table_name(), |query| {
                    self.build_query(query, without_trash);
                    if !email.is_empty() {
                        query.eq(
                            UserEntity::prefix_with_tbl(UserEntity::col_name_for_email()),
                            email,
                        );
                    } else {
                        query.eq(
                            UserEntity::prefix_with_tbl(UserEntity::col_name_for_username()),
                            username,
                        );
                    }
                })
                .fetch_all()
                .await
                .map(|list| self.build_entity_result(list.expect("user not found")))
        } else {
            Err(anyhow::anyhow!("Both username and email values are empty"))
        }
    }

    // pub async fn find_by_id_and_salt(&self, user_id: &str, salt: &str) -> Result {

    // }

    pub async fn get_user_logged_in_info(
        &self,
        user_id: &str,
    ) -> Result<LoggedInUser, anyhow::Error> {
        self.manager
            .select_from_table(DirtybaseUserEntity::table_name(), |query| {
                self.build_query(query, true);
                query.eq(DirtybaseUserEntity::user_id_column(), user_id);
            })
            .fetch_all()
            .await
            .map(|list| {
                if let Some(result) = list {
                    self.build_dto_result(result)
                } else {
                    self.build_dto_result(list.unwrap())
                }
            })
    }

    pub async fn find_by_core_user_id(
        &self,
        core_user_id: &str,
    ) -> Result<Option<DirtybaseUserEntity>, anyhow::Error> {
        self.manager
            .select_from_table(DirtybaseUserEntity::table_name(), |q| {
                q.eq(UserEntity::foreign_id_column().unwrap(), core_user_id);
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: DirtybaseUserEntity,
    ) -> Result<Option<DirtybaseUserEntity>, anyhow::Error> {
        let core_user_id = record.core_user_id.clone();

        self.manager
            .insert(DirtybaseUserEntity::table_name(), record)
            .await;

        self.find_by_core_user_id(&core_user_id).await
    }

    pub async fn update(&self, record: DirtybaseUserEntity) -> Result<bool, anyhow::Error> {
        let core_user_id = record.core_user_id.clone();
        let fields = record.into_column_value();
        self.manager
            .update(DirtybaseUserEntity::table_name(), fields, |query| {
                query.eq(
                    DirtybaseUserEntity::col_name_for_core_user_id(),
                    &core_user_id,
                );
            })
            .await;

        Ok(true)
    }

    fn build_dto_result(&self, list: Vec<StructuredColumnAndValue>) -> LoggedInUser {
        let mut base = LoggedInUser::default();
        let mut apps: HashMap<String, UserAppDto> = HashMap::new();

        for mut entry in list.into_iter().enumerate() {
            if entry.0 == 0 {
                base = LoggedInUser::from_struct_column_value(&mut entry.1.clone(), None).unwrap();
                base.is_sys_admin = self.is_sys_admin(entry.1.get(SYS_ADMIN_JOIN_PREFIX));
            }

            self.build_app_map(&mut apps, &mut entry.1);
        }

        base.apps = apps.into_iter().map(|e| e.1).collect();

        base
    }

    fn build_entity_result(&self, list: Vec<StructuredColumnAndValue>) -> DirtybaseUserEntity {
        let mut base = DirtybaseUserEntity::default();
        let mut apps: HashMap<String, UserAppDto> = HashMap::new();

        for mut entry in list.into_iter().enumerate() {
            if entry.0 == 0 {
                base = DirtybaseUserEntity::from_struct_column_value(&mut entry.1, None).unwrap();
                base.is_sys_admin = self.is_sys_admin(entry.1.get(SYS_ADMIN_JOIN_PREFIX));
            }

            self.build_app_map(&mut apps, &mut entry.1);
        }

        base.apps = apps.into_iter().map(|e| e.1).collect();
        base
    }

    fn build_query(&self, query: &mut QueryBuilder, without_trash: bool) {
        if without_trash {
            query.without_table_trash::<UserEntity>();
        }
        query
            .select_multiple(DirtybaseUserEntity::table_column_full_names())
            .left_join_table_and_select::<UserEntity, DirtybaseUserEntity>(
                UserEntity::id_column().unwrap(),
                UserEntity::foreign_id_column().unwrap(),
                Some(USER_JOIN_PREFIX),
            )
            .left_join_table_and_select::<SysAdminEntity, UserEntity>(
                UserEntity::foreign_id_column().unwrap(),
                UserEntity::id_column().unwrap(),
                Some(SYS_ADMIN_JOIN_PREFIX),
            )
            .left_join_table::<RoleUserEntity, UserEntity>(
                RoleUserEntity::role_user_fk_column(),
                UserEntity::id_column().unwrap(),
            )
            .left_join_table_and_select::<RoleEntity, RoleUserEntity>(
                RoleEntity::id_column().unwrap(),
                RoleUserEntity::app_role_fk_column(),
                Some(ROLES_JOIN_PREFIX),
            )
            .left_join_table_and_select::<AppEntity, RoleEntity>(
                AppEntity::id_column().unwrap(),
                AppEntity::foreign_id_column().unwrap(),
                Some(APPS_JOIN_PREFIX),
            )
            .left_join_table_and_select::<CompanyEntity, AppEntity>(
                CompanyEntity::id_column().unwrap(),
                CompanyEntity::foreign_id_column().unwrap(),
                Some(COMPANY_JOIN_PREFIX),
            )
            .without_table_trash::<AppEntity>()
            .without_table_trash::<CompanyEntity>()
            .without_table_trash::<RoleUserEntity>();
    }

    fn is_sys_admin(&self, field_value: Option<&FieldValue>) -> bool {
        if let Some(FieldValue::Object(obj)) = field_value {
            if let Some(value) = obj.get(SysAdminEntity::col_name_for_core_user_id()) {
                matches!(value, FieldValue::String(_))
            } else {
                false
            }
        } else {
            false
        }
    }

    fn build_app_map(
        &self,
        apps: &mut HashMap<String, UserAppDto>,
        data: &mut StructuredColumnAndValue,
    ) {
        if let Some(FieldValue::Object(app_obj)) = data.get(APPS_JOIN_PREFIX) {
            let id = if let Some(i) = app_obj.get("id") {
                i.to_string()
            } else {
                "".into()
            };

            if apps.get(&id).is_none() {
                apps.insert(id.clone(), UserAppDto::from_column_value(app_obj.clone()));
            }

            if app_obj.contains_key("roles") {
                apps.get_mut(&id)
                    .unwrap()
                    .roles
                    .push(app_obj.get("roles").unwrap().into());
            }
        }
    }
}

#[busybody::async_trait]
impl busybody::Injectable for DirtybaseUserRepository {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app = ci.get::<App>().await.unwrap();

        Self::new(app.schema_manger())
    }
}
