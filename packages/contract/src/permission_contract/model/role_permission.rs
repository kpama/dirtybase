use crate::db_contract::types::{ArcUuid7, ColumnAndValue, FromColumnAndValue};

use super::{actor::Actor, permission::Permission, role::Role, PermissionRelAction};

pub struct RolePermission {
    role_id: Option<ArcUuid7>, // The permission could be applied directly on the actor
    actor_id: Option<ArcUuid7>, // The permission could be applied to one of the actor's roles. This is the recommended way
    tenant_id: Option<ArcUuid7>,
    permission_id: ArcUuid7,
    actor: Option<Actor>,
    role: Option<Role>,
    permission: Option<Permission>,
}

#[derive(Debug, Clone)]
pub enum PermissionSubject {
    Role(ArcUuid7),
    Actor(ArcUuid7),
}

pub struct RolePermissionPayload {
    pub rel_action: PermissionRelAction,
    pub subject: PermissionSubject,
    pub permission_id: ArcUuid7,
}

impl RolePermission {
    pub fn role_id(&self) -> Option<ArcUuid7> {
        self.role_id.clone()
    }

    pub fn actor_id(&self) -> Option<ArcUuid7> {
        self.actor_id.clone()
    }

    pub fn tenant_id(&self) -> Option<ArcUuid7> {
        self.tenant_id.clone()
    }

    pub fn permission_id(&self) -> ArcUuid7 {
        self.permission_id.clone()
    }

    pub fn actor(&self) -> Option<Actor> {
        self.actor.clone()
    }

    pub fn role(&self) -> Option<Role> {
        self.role.clone()
    }

    pub fn permission(&self) -> Option<Permission> {
        self.permission.clone()
    }
}

impl FromColumnAndValue for RolePermission {
    fn from_column_value(
        mut cv: crate::db_contract::types::ColumnAndValue,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let record = Self {
            role_id: cv.remove("role_id").map(ArcUuid7::from),
            actor_id: cv.remove("actor_id").map(ArcUuid7::from),
            tenant_id: cv.remove("tenant_id").map(ArcUuid7::from),
            permission_id: cv
                .remove("permission_id")
                .ok_or(anyhow::anyhow!("actor_id not set for actor role"))?
                .into(),
            actor: if let Some(actor) = cv.remove("actor") {
                Actor::from_column_value(ColumnAndValue::from(actor)).ok()
            } else {
                None
            },
            role: if let Some(actor) = cv.remove("role") {
                Role::from_column_value(ColumnAndValue::from(actor)).ok()
            } else {
                None
            },
            permission: if let Some(actor) = cv.remove("permission") {
                Permission::from_column_value(ColumnAndValue::from(actor)).ok()
            } else {
                None
            },
        };

        if !cv.is_empty() {
            return Err(anyhow::anyhow!(
                "one or more role permission column values were not handled: {:#?}",
                cv.keys().collect::<Vec<&String>>()
            ));
        }

        Ok(record)
    }
}
