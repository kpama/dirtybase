use std::{collections::HashMap, sync::Arc};

use serde::de::DeserializeOwned;

use crate::db::types::ArcUuid7;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TenantContext {
    id: ArcUuid7,
    config: Arc<HashMap<String, String>>,
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

    /// Returns a JSON representation of the configuration
    pub fn config_string(&self, key: &str) -> Option<String> {
        self.config.get(key).cloned()
    }

    /// Tries to parse the JSON to the specified type
    pub fn config_to<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        if let Some(s) = self.config_string(key) {
            serde_json::from_str(&s).ok()
        } else {
            None
        }
    }
}
