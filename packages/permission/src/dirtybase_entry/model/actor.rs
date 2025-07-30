use std::fmt::Display;

use dirtybase_contract::db_contract::types::{ArcUuid7, DateTimeField, NameField};
use dirtybase_db_macro::DirtyTable;
use dirtybase_helper::time::current_datetime;
use serde::{Deserialize, Serialize};

use crate::dirtybase_entry::model::{permission::Permission, role::Role};

#[derive(Debug, Clone, Default, Serialize, Deserialize, DirtyTable)]
pub struct Actor {
    id: Option<ArcUuid7>,
    user_id: Option<ArcUuid7>,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
    #[dirty(rel(kind = has_many))]
    roles: Vec<Role>,
    #[dirty(rel(kind = has_many))]
    permissions: Vec<Permission>,
}

impl Actor {
    // TODO: This should be handled by the model repository
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

impl Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.id.as_ref() {
            Some(id) => write!(f, "actor:{id}"),
            None => write!(f, "actor:"),
        }
    }
}
