use crate::db::types::ArcUuid7;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RoleContext {
    id: ArcUuid7,
    is_global: bool,
}

impl RoleContext {
    pub fn make_global() -> Self {
        Self {
            id: ArcUuid7::default(),
            is_global: true,
        }
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }
}
