use std::{collections::HashMap, sync::Arc};

use serde::de::DeserializeOwned;

use crate::db::types::ArcUuid7;

pub const GLOBAL_TENANT_CONTEXT_ID: &str = "0194d472-8475-7791-9158-f056ad78cdac";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TenantContext {
    id: ArcUuid7,
    config: Arc<HashMap<String, String>>,
}

impl TenantContext {
    pub fn make_global() -> Self {
        Self {
            id: ArcUuid7::try_from(GLOBAL_TENANT_CONTEXT_ID).unwrap(),
            ..Default::default()
        }
    }
    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn is_global(&self) -> bool {
        self.id.to_string() == GLOBAL_TENANT_CONTEXT_ID
    }

    /// Returns a json representation of the configuration
    pub fn config_string(&self, key: &str) -> Option<String> {
        self.config.get(key).cloned()
    }

    /// Tries to parse the json to the specified type
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
