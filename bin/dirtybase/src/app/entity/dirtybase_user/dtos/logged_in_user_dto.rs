use dirtybase_db::entity::user::{UserEntity, UserStatus};

/// Outgoing DTO when the user successfully logged in
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LoggedInUser {
    pub username: String,
    pub reset_password: bool,
    pub status: String,
    pub token: Option<String>,
}

impl Default for LoggedInUser {
    fn default() -> Self {
        Self {
            username: "".into(),
            reset_password: false,
            status: UserStatus::Pending.to_string(),
            token: None,
        }
    }
}

impl From<UserEntity> for LoggedInUser {
    fn from(value: UserEntity) -> Self {
        Self {
            username: value.username.unwrap_or_default(),
            reset_password: value.reset_password.unwrap_or_default(),
            status: value.status.unwrap_or_default().to_string(),
            token: None,
        }
    }
}
