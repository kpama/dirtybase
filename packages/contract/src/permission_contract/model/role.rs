use std::sync::Arc;

use dirtybase_helper::time::current_datetime;
use serde::{Deserialize, Serialize};

use crate::db_contract::types::{ArcUuid7, DateTimeField, NameField, OptionalNameField};

use super::PermissionRecordAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    id: ArcUuid7,
    name: NameField,
    label: Arc<String>,
    description: Arc<String>,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
}

impl Role {
    pub fn touch_created_at(&mut self) {
        self.created_at = current_datetime().into();
    }

    pub fn touch_updated_at(&mut self) {
        self.updated_at = current_datetime().into();
    }

    pub fn touch_deleted_at(&mut self) {
        self.deleted_at = current_datetime().into()
    }
}

pub struct RolePayload {
    action: Option<PermissionRecordAction>,
    id: Option<ArcUuid7>,
    name: OptionalNameField,
}
