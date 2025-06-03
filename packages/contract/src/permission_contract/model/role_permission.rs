use crate::db_contract::types::ArcUuid7;

use super::{actor::Actor, permission::Permission, role::Role, PermissionRelAction};

pub struct RolePermission {
    role_id: Option<ArcUuid7>, // The permission could be applied directly on the actor
    actor_id: Option<ArcUuid7>, // The permission could be applied to one of the actor's roles. This is the recommended way
    tenant_id: Option<ArcUuid7>,
    permission_id: ArcUuid7,
    actor: Option<Actor>,
    role: Option<Role>,
    permission: Permission,
}

#[derive(Debug, Clone)]
pub enum PermissionSubject {
    Role(ArcUuid7),
    Actor(ArcUuid7),
}

pub struct RolePermissionPayload {
    action: PermissionRelAction,
    subject: PermissionSubject,
    permission_id: ArcUuid7,
}
