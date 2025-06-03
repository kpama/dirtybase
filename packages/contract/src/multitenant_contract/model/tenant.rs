use std::sync::Arc;

use crate::db_contract::types::{ArcUuid7, DateTimeField};

pub struct Tenant {
    id: ArcUuid7,
    name: Arc<String>,
    domain: Option<Arc<String>>,
    is_global: bool,
    created_at: DateTimeField,
    updated_at: DateTimeField,
    deleted_at: DateTimeField,
}
