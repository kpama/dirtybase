mod request_tenant_resolver;
mod storage;
mod tenant_context;
mod tenant_repository;

pub mod model;

use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use dirtybase_common::db::field_values::FieldValue;
pub use request_tenant_resolver::*;
use serde::{Deserialize, Serialize};
pub use tenant_context::*;
pub use tenant_repository::*;

use crate::db_contract::{
    // field_values::FieldValue,
    types::ArcUuid7,
};

#[derive(Default, Clone, Serialize, Deserialize)]
struct TenantId(ArcUuid7);

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
