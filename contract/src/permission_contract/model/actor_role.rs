use dirtybase_common::db::types::{ArcUuid7, DateTimeField, StatusField};
use dirtybase_db_macro::DirtyTable;
use serde::{Deserialize, Serialize};

use super::{actor::Actor, role::Role};

#[derive(Debug, Clone, Default, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "perm_actor_roles")]
pub struct ActorRole {
    pub(crate) id: Option<ArcUuid7>,
    perm_actor_id: ArcUuid7,
    perm_role_id: ArcUuid7,
    status: StatusField,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
    #[dirty(rel(kind = "belongs_to"))]
    actor: Option<Actor>,
    #[dirty(rel(kind = "belongs_to"))]
    role: Option<Role>,
}

impl ActorRole {
    pub fn new(actor_id: ArcUuid7, role_id: ArcUuid7) -> Self {
        Self {
            id: Some(ArcUuid7::default()),
            perm_actor_id: actor_id,
            perm_role_id: role_id,
            ..Default::default()
        }
    }

    pub fn id(&self) -> Option<&ArcUuid7> {
        self.id.as_ref()
    }

    pub fn actor_id(&self) -> &ArcUuid7 {
        &self.perm_actor_id
    }

    pub fn set_actor_id(&mut self, actor_id: ArcUuid7) -> &mut Self {
        self.perm_actor_id = actor_id.into();
        self
    }

    pub fn role_id(&self) -> &ArcUuid7 {
        &self.perm_role_id
    }

    pub fn set_role_id(&mut self, role_id: ArcUuid7) -> &mut Self {
        self.perm_role_id = role_id.into();
        self
    }

    pub fn actor(&self) -> Option<&Actor> {
        self.actor.as_ref()
    }

    pub fn role(&self) -> Option<&Role> {
        self.role.as_ref()
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
pub enum PersistActorRolePayload {
    Save { record: ActorRole },
    Delete { id: ArcUuid7 },
    Restore { id: ArcUuid7 },
    Destroy { id: ArcUuid7 },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FetchActorRoleOption {
    pub with_trashed: bool,
    pub with_role: bool,
    pub with_actor: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchActorRolePayload {
    ById {
        id: ArcUuid7,
    },

    ByActorAndRole {
        actor_id: ArcUuid7,
        role_id: ArcUuid7,
    },
}
