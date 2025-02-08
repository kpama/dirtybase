use crate::db::types::ArcUuid7;

pub const GLOBAL_TENANT_CONTEXT_ID: &str = "0194d472-8475-7791-9158-f056ad78cdac";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TenantContext {
    id: ArcUuid7,
}

impl TenantContext {
    pub fn make_global() -> Self {
        Self {
            id: ArcUuid7::from(GLOBAL_TENANT_CONTEXT_ID),
        }
    }
    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn is_global(&self) -> bool {
        self.id.to_string() == GLOBAL_TENANT_CONTEXT_ID
    }
}
