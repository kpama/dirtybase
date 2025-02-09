use std::{collections::HashMap, sync::Arc};

use serde::de::DeserializeOwned;

use crate::db::types::ArcUuid7;

pub const GLOBAL_APP_CONTEXT_ID: &str = "0194d479-fee1-7a81-8c20-5b8726efddf0";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppContext {
    id: ArcUuid7,
    /// Extensions configurations in JSON format
    config_cache: Arc<HashMap<String, String>>,
}

impl AppContext {
    pub fn make_global() -> Self {
        Self {
            id: ArcUuid7::try_from(GLOBAL_APP_CONTEXT_ID).unwrap(),
            ..Default::default()
        }
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    /// Returns a json representation of the configuration
    pub fn config_string(&self, key: &str) -> Option<String> {
        self.config_cache.get(key).cloned()
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
