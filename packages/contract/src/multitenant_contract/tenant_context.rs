use crate::db_contract::types::ArcUuid7;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TenantContext {
    id: ArcUuid7,
    is_global: bool,
}

impl Default for TenantContext {
    fn default() -> Self {
        Self {
            is_global: true,
            id: ArcUuid7::default(),
        }
    }
}

impl TenantContext {
    pub fn make_global() -> Self {
        Self::default()
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn id_as_string(&self) -> String {
        self.id.to_string()
    }

    pub fn id_as_uuid25_string(&self) -> String {
        self.id.to_uuid25_string()
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }
}
