use crate::db_contract::types::ArcUuid7;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TenantContext {
    id: ArcUuid7,
    is_global: bool,
}

impl TenantContext {
    pub fn make_global() -> Self {
        Self {
            is_global: true,
            ..Default::default()
        }
    }
    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }
}
