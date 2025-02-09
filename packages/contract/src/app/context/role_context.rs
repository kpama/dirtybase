use crate::db::types::ArcUuid7;

pub const GLOBAL_ROLE_CONTEXT_ID: &str = "0194d475-f52d-7190-b47c-f90ab8916f02";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RoleContext {
    id: ArcUuid7,
}

impl RoleContext {
    pub fn make_global() -> Self {
        Self {
            id: ArcUuid7::try_from(GLOBAL_ROLE_CONTEXT_ID).unwrap(),
        }
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn is_global(&self) -> bool {
        self.id.to_string() == GLOBAL_ROLE_CONTEXT_ID
    }
}
