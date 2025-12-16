use dirtybase_db_macro::DirtyTable;
use serde::{Deserialize, Serialize};

use crate::{
    auth_contract::AuthUserStatus,
    db_contract::types::{ArcUuid7, DateTimeField, StringField},
};

pub type TenantId = ArcUuid7;

pub type TenantStatus = AuthUserStatus;

#[derive(Debug, Default, Clone, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "perm_tenants")]
pub struct Tenant {
    id: Option<TenantId>,
    name: StringField,
    domain: Option<StringField>,
    is_global: bool,
    status: TenantStatus,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
}

impl Tenant {
    pub fn id(&self) -> Option<&TenantId> {
        self.id.as_ref()
    }

    pub fn name(&self) -> &StringField {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string().into();
        self
    }

    pub fn domain(&self) -> Option<&StringField> {
        self.domain.as_ref()
    }

    pub fn set_domain(&mut self, domain: &str) -> &mut Self {
        self.domain = Some(domain.to_string().into());
        self
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }

    pub fn status(&self) -> &TenantStatus {
        &self.status
    }

    pub fn set_status(&mut self, status: TenantStatus) -> &mut Self {
        self.status = status;
        self
    }

    pub fn created_at(&self) -> Option<&DateTimeField> {
        self.created_at.as_ref()
    }

    pub fn updated_at(&self) -> Option<&DateTimeField> {
        self.updated_at.as_ref()
    }

    pub fn deleted_at(&self) -> Option<&DateTimeField> {
        self.deleted_at.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersistTenantPayload {
    Save { tenant: Tenant },
    Delete { id: TenantId },
    Restore { id: TenantId },
    Destroy { id: TenantId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchTenantOption {
    // TODO: No options for now
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchTenantPayload {
    ById { id: TenantId },
    ByName { name: StringField },
    ByDomain { name: StringField },
}
