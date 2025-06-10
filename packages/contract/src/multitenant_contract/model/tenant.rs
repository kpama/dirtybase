use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::db_contract::types::{ArcUuid7, DateTimeField, StringField};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Tenant {
    id: ArcUuid7,
    name: StringField,
    domain: Option<Arc<String>>,
    is_global: bool,
    created_at: DateTimeField,
    updated_at: DateTimeField,
    deleted_at: DateTimeField,
}
