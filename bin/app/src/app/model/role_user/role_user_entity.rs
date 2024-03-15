use crate::app::model::role::RoleEntity;
use dirtybase_contract::db::{entity::user::UserEntity, macros::DirtyTable};
use dirtybase_contract::db::{
    types::{DateTimeField, UlidField},
    TableEntityTrait,
};

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_role_user")]
pub struct RoleUserEntity {
    pub core_app_role_id: UlidField,
    pub core_user_id: UlidField,
    pub creator_id: UlidField,
    pub editor_id: UlidField,
    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}

impl RoleUserEntity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn app_role_fk_column() -> &'static str {
        RoleEntity::foreign_id_column().unwrap()
    }

    pub fn role_user_fk_column() -> &'static str {
        UserEntity::foreign_id_column().unwrap()
    }
}
