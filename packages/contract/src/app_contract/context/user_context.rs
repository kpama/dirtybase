use crate::{db_contract::types::ArcUuid7, user::status::UserStatus};

pub const GLOBAL_USER_CONTEXT_ID: &str = "0194d471-0c6d-75f2-b234-03343345edbc";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UserContext {
    id: ArcUuid7,
    status: UserStatus,
}

impl UserContext {
    pub fn make_global() -> Self {
        Self {
            id: ArcUuid7::try_from(GLOBAL_USER_CONTEXT_ID).unwrap(),
            status: UserStatus::Active,
        }
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn status(&self) -> UserStatus {
        self.status.clone()
    }

    pub fn status_ref(&self) -> &UserStatus {
        &self.status
    }

    pub fn is_global(&self) -> bool {
        self.id.to_string() == GLOBAL_USER_CONTEXT_ID
    }
}
