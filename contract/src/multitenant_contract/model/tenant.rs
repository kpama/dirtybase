use std::fmt::{Debug, Display};
use std::ops::Deref;

use dirtybase_common::db::field_values::FieldValue;
use dirtybase_db_macro::DirtyTable;
use serde::{Deserialize, Serialize};

use crate::{
    auth_contract::AuthUserStatus,
    db_contract::types::{ArcUuid7, DateTimeField, StringField},
};

pub type TenantStatus = AuthUserStatus;

#[derive(Default, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantId(ArcUuid7);

impl TenantId {
    pub fn new() -> Self {
        Self(ArcUuid7::default())
    }
}

impl Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Deref for TenantId {
    type Target = ArcUuid7;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ArcUuid7> for TenantId {
    fn from(value: ArcUuid7) -> Self {
        Self(value)
    }
}

impl From<&ArcUuid7> for TenantId {
    fn from(value: &ArcUuid7) -> Self {
        Self(value.clone())
    }
}

impl From<TenantId> for ArcUuid7 {
    fn from(value: TenantId) -> Self {
        value.0
    }
}

impl From<&TenantId> for ArcUuid7 {
    fn from(value: &TenantId) -> Self {
        value.0.clone()
    }
}

impl From<FieldValue> for TenantId {
    fn from(value: FieldValue) -> Self {
        TenantId(value.into())
    }
}

impl From<&FieldValue> for TenantId {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
    }
}

impl From<TenantId> for FieldValue {
    fn from(value: TenantId) -> Self {
        value.0.into()
    }
}

impl From<&TenantId> for FieldValue {
    fn from(value: &TenantId) -> Self {
        value.clone().into()
    }
}

impl TryFrom<&str> for TenantId {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let id = match ArcUuid7::try_from(value) {
            Ok(id) => id,
            Err(e) => return Err(anyhow::anyhow!(e)),
        };
        Ok(Self(id))
    }
}

impl TryFrom<String> for TenantId {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let id = match ArcUuid7::try_from(value.as_str()) {
            Ok(id) => id,
            Err(e) => return Err(anyhow::anyhow!(e)),
        };
        Ok(Self(id))
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "tenants")]
pub struct Tenant {
    id: Option<TenantId>,
    name: StringField,
    token: StringField,
    domain: Option<StringField>,
    status: TenantStatus,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
}

impl Tenant {
    pub fn new() -> Self {
        let id = TenantId::new();
        let token = id.0.to_uuid25_string();
        Self {
            id: Some(id),
            token: token.into(),
            ..Default::default()
        }
    }

    pub fn id(&self) -> Option<&TenantId> {
        self.id.as_ref()
    }

    pub fn name(&self) -> &StringField {
        &self.name
    }

    pub fn token(&self) -> &StringField {
        &self.token
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
    Delete { tenant: Tenant },
    Restore { tenant: Tenant },
    Destroy { tenant: Tenant },
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FetchTenantOption {
    with_trached: bool,
    status: TenantStatus,
}

impl FetchTenantOption {
    pub fn with_status(status: TenantStatus) -> Self {
        let mut opt = Self::default();
        opt.status = status;
        opt
    }
    pub fn only_active() -> Self {
        Self::with_status(TenantStatus::Active)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchTenantPayload {
    ById { id: TenantId },
    ByName { name: StringField },
    ByToken { token: StringField },
    ByDomain { domain: StringField },
}

impl FetchTenantPayload {
    pub fn by_id(id: TenantId) -> Self {
        Self::ById { id }
    }

    pub fn by_name<T: ToString>(name: T) -> Self {
        Self::ByName {
            name: name.to_string().into(),
        }
    }

    pub fn by_token<T: ToString>(token: T) -> Self {
        Self::ByToken {
            token: token.to_string().into(),
        }
    }

    pub fn by_domain<T: ToString>(domain: T) -> Self {
        Self::ByDomain {
            domain: domain.to_string().into(),
        }
    }
}
