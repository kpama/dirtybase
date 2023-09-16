use super::out_user_app::UserAppDto;
use crate::app::model::dirtybase_user::DirtybaseUserEntity;
use dirtybase_db::{entity::user::dtos::OutUserEntityDto, macros::DirtyTable};
use dirtybase_db_types::types::{StructuredColumnAndValue, TimestampField};

/// Outgoing DTO when the user successfully logged in
#[derive(Debug, Default, serde::Serialize, Clone, DirtyTable)]
pub struct LoggedInUser {
    pub login_attempt: i64,
    pub last_login_at: TimestampField,
    pub token: String,
    pub apps: Vec<UserAppDto>,
    pub user: OutUserEntityDto,
    pub is_sys_admin: bool,
}

impl LoggedInUser {
    pub fn append_from_structured(&mut self, cv: &mut StructuredColumnAndValue) {
        if let Some(app) = UserAppDto::from_struct_column_value(cv, Some("app")) {
            self.apps.push(app);
        }
    }
}

impl From<DirtybaseUserEntity> for LoggedInUser {
    fn from(value: DirtybaseUserEntity) -> Self {
        Self {
            login_attempt: value.login_attempt,
            token: "".to_owned(),
            apps: value.apps.into_iter().map(|i| i.into()).collect(),
            user: value.user.into(),
            is_sys_admin: value.is_sys_admin,
            last_login_at: value.last_login_at,
        }
    }
}
