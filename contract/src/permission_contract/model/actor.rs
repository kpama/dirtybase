use std::fmt::Display;

use super::{ActorRole, Permission, Role, RolePermission};
use dirtybase_common::db::types::{ArcUuid7, DateTimeField};
use dirtybase_db_macro::DirtyTable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "perm_actors")]
pub struct Actor {
    id: Option<ArcUuid7>,
    user_id: Option<ArcUuid7>,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
    #[dirty(rel(kind = "has_many_through", pivot: ActorRole))]
    roles: Vec<Role>,
    #[dirty(rel(kind = "has_many_through", pivot: RolePermission))]
    permissions: Vec<Permission>,
}

impl Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.id.as_ref() {
            Some(id) => write!(f, "actor:{id}"),
            None => write!(f, "actor:"),
        }
    }
}

impl Actor {
    pub fn id(&self) -> Option<&ArcUuid7> {
        self.id.as_ref()
    }

    pub fn set_id(&mut self, id: ArcUuid7) -> &mut Self {
        self.id = Some(id);
        self
    }

    pub fn user_id(&self) -> Option<&ArcUuid7> {
        self.user_id.as_ref()
    }

    pub fn set_user_id(&mut self, user_id: ArcUuid7) -> &mut Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn roles(&self) -> &[Role] {
        &self.roles
    }

    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }

    pub fn created_at(&self) -> Option<&DateTimeField> {
        self.created_at.as_ref()
    }
    pub fn update_at(&self) -> Option<&DateTimeField> {
        self.updated_at.as_ref()
    }

    pub fn deleted_at(&self) -> Option<&DateTimeField> {
        self.deleted_at.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersistActorPayload {
    Save { actor: Actor },
    Delete { id: ArcUuid7 },
    Restore { id: ArcUuid7 },
    Destroy { id: ArcUuid7 },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FetchActorOption {
    pub check_trashed: bool,
    pub with_roles: bool,
    pub with_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchActorPayload {
    ById { id: ArcUuid7 },
    ByUserId { user_id: ArcUuid7 },
}
