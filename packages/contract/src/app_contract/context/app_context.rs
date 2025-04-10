use std::{collections::HashMap, sync::Arc};

use crate::db_contract::types::ArcUuid7;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppContext {
    id: ArcUuid7,
    is_global: bool,
    config_store: Arc<HashMap<String, String>>,
}

impl AppContext {
    pub fn make_global() -> Self {
        Self {
            id: ArcUuid7::default(),
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

    /// Returns a JSON representation of the configuration
    pub async fn config_string(&self, key: &str) -> Option<String> {
        self.config_store.get(key).cloned()
    }
}
