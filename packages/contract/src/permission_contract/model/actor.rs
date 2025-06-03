use dirtybase_helper::time::current_datetime;
use serde::{Deserialize, Serialize};

use crate::db_contract::types::{ArcUuid7, DateTimeField};

use super::{permission::Permission, role::Role, PermissionRecordAction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    id: Option<ArcUuid7>,
    user_id: Option<ArcUuid7>,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
    roles: Option<Vec<Role>>,
    permissions: Option<Vec<Permission>>,
}

pub struct ActorPayload {
    action: Option<PermissionRecordAction>,
    id: Option<ArcUuid7>,
    user_id: Option<ArcUuid7>,
}

impl Actor {
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
