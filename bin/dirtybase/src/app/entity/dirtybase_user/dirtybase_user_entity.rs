use super::dtos::out_user_app::UserAppDto;
use dirtybase_db::{
    base::helper::generate_ulid,
    dirtybase_db_types::types::{DateTimeField, UlidField},
    dirtybase_db_types::TableEntityTrait,
    entity::user::UserEntity,
    macros::DirtyTable,
};
use dirtybase_db_types::types::StructuredColumnAndValue;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_dirtybase_user")]
pub struct DirtybaseUserEntity {
    pub core_user_id: UlidField,
    pub login_attempt: u64,
    pub last_login_at: DateTimeField,
    pub salt: String,
    #[dirty(skip_select, skip_insert)]
    pub apps: Vec<UserAppDto>,
    #[dirty(skip_select, skip_insert)]
    pub user: UserEntity,
    #[dirty(skip)]
    pub is_sys_admin: bool,
}

impl DirtybaseUserEntity {
    pub fn user_id_column() -> String {
        format!(
            "{}",
            Self::prefix_with_tbl(UserEntity::foreign_id_column().unwrap())
        )
    }

    pub fn append_from_structured(&mut self, cv: &mut StructuredColumnAndValue) {
        let mut apps = Vec::new();
        if let Some(app) = UserAppDto::from_struct_column_value(cv, Some("app")) {
            apps.push(app);
        }
    }

    pub fn append_app(&mut self, cv: &mut StructuredColumnAndValue) -> Vec<UserAppDto> {
        let mut apps = Vec::new();
        if let Some(app) = UserAppDto::from_struct_column_value(cv, Some("app")) {
            apps.push(app);
        }

        return apps;
    }

    pub fn generate_salt(&mut self) {
        let mut salt_hasher = Sha256::new();
        salt_hasher.update(generate_ulid().as_bytes());
        self.salt = format!("{:x}", salt_hasher.finalize());
    }
}
