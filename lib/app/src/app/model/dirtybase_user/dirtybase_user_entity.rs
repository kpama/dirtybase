use super::dtos::out_user_app::UserAppDto;
use dirtybase_contract::db::{
    base::helper::generate_ulid,
    entity::user::UserEntity,
    macros::DirtyTable,
    types::{StructuredColumnAndValue, TimestampField, UlidField},
};
use dirtybase_db::TableEntityTrait;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_dirtybase_user")]
pub struct DirtybaseUserEntity {
    pub core_user_id: UlidField,
    pub login_attempt: i64,
    pub last_login_at: TimestampField,
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
        let mut hash = Sha256::new();
        hash.update(generate_ulid().as_bytes());
        self.salt = format!("{:x}", hash.finalize());
    }

    pub fn reflect_login_success(&mut self) {
        self.last_login_at = Some(chrono::Utc::now());
        self.reset_login_attempts();
    }

    pub fn reset_login_attempts(&mut self) {
        self.login_attempt = 0;
    }

    pub fn reflect_login_failure(&mut self) {
        self.login_attempt += 1;
    }
}
