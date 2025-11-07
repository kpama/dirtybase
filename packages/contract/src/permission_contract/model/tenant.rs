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
