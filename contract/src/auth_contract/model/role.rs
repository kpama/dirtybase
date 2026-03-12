use dirtybase_common::db::types::{ArcUuid7, DateTimeField, LabelField, NameField, StringField};
use serde::{Deserialize, Serialize};

use dirtybase_db_macro::DirtyTable;

use crate::auth_contract::{Actor, ActorRole, Permission, RolePermission};

#[derive(Debug, Clone, Default, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "auth_roles", id_not_auto, timestamp, soft_deletable)]
pub struct Role {
    pub(crate) id: Option<ArcUuid7>,
    name: NameField,
    label: LabelField,
    description: StringField,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
    #[dirty(rel(kind = "has_many_through", pivot = RolePermission, soft_deletable))]
    permissions: Vec<Permission>,
    #[dirty(rel(kind = "has_many", soft_deletable))]
    actor_roles: Vec<ActorRole>,
    #[dirty(rel(kind = "has_many_through", pivot = ActorRole, soft_deletable))]
    actors: Vec<Actor>,
}

impl Role {
    pub fn new(name: &str, label: &str) -> Self {
        Self {
            id: Some(ArcUuid7::default()),
            name: name.to_string().into(),
            label: label.to_string().into(),
            ..Default::default()
        }
    }

    pub fn create_guest() -> Self {
        Self {
            id: Some(ArcUuid7::default()),
            name: "guest".to_string().into(),
            label: "Guest Role".to_string().into(),
            ..Default::default()
        }
    }

    pub fn actor_roles(&self) -> &[ActorRole] {
        &self.actor_roles
    }

    pub fn actors(&self) -> &[Actor] {
        &self.actors
    }

    pub fn id(&self) -> Option<&ArcUuid7> {
        self.id.as_ref()
    }

    pub fn set_id(&mut self, id: ArcUuid7) -> &mut Self {
        self.id = Some(id);
        self
    }

    pub fn name(&self) -> &NameField {
        &self.name
    }

    pub fn set_name(&mut self, name: NameField) -> &mut Self {
        self.name = name;
        self
    }

    pub fn label(&self) -> &LabelField {
        &self.label
    }

    pub fn set_label(&mut self, label: LabelField) -> &mut Self {
        self.label = label;
        self
    }

    pub fn description(&self) -> &StringField {
        &self.description
    }

    pub fn set_description(&mut self, description: &str) -> &mut Self {
        self.description = description.to_string().into();
        self
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

    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersistRolePayload {
    Save { role: Role },
    Delete { id: ArcUuid7 },
    Restore { id: ArcUuid7 },
    Destroy { id: ArcUuid7 },
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FetchRoleOption {
    pub with_trashed: bool,
    pub with_permissions: bool,
    pub with_actors: bool,
    pub with_actor_roles: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchRolePayload {
    ById { id: ArcUuid7 },
    ByName { name: NameField },
}

impl FetchRolePayload {
    pub fn by_name(name: &str) -> Self {
        Self::ByName { name: name.into() }
    }

    pub fn by_id(id: ArcUuid7) -> Self {
        Self::ById { id }
    }
}
