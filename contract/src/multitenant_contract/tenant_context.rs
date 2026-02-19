use std::{collections::HashMap, sync::Arc};

use crate::{db_contract::types::ArcUuid7, multitenant_contract::model::Tenant};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TenantContext {
    id: ArcUuid7,
    is_global: bool,
    tenant: Option<Tenant>,
    config_store: Arc<HashMap<String, String>>,
}

impl Default for TenantContext {
    fn default() -> Self {
        Self {
            is_global: false,
            id: ArcUuid7::default(),
            tenant: None,
            config_store: Arc::default(),
        }
    }
}

impl TenantContext {
    pub fn new(tenant: Tenant, config_store: HashMap<String, String>) -> Self {
        let mut ctx = Self::default();
        ctx.id = tenant.id().cloned().unwrap_or_default().into();
        ctx.tenant = Some(tenant);
        ctx.config_store = config_store.into();
        ctx
    }

    pub fn make_global() -> Self {
        let mut ctx = Self::default();
        ctx.is_global = true;
        ctx
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn tenant_name(&self) -> &str {
        if let Some(t) = &self.tenant {
            return t.name();
        }
        "--global tenant--"
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

    pub fn has_tenant(&self) -> bool {
        self.tenant.is_some()
    }

    pub fn tenant(&self) -> Option<&Tenant> {
        self.tenant.as_ref()
    }

    pub async fn config_string(&self, key: &str) -> Option<&String> {
        self.config_store.get(key)
    }
}
