use crate::{
    db_contract::types::ArcUuid7,
    multitenant_contract::{model::Tenant, TenantId},
};

use super::{actor::Actor, role::Role, PermissionRelAction};

pub struct ActorRole {
    actor_id: ArcUuid7,
    role_id: ArcUuid7,
    tenant_id: Option<TenantId>,
    actor: Option<Actor>,
    role: Option<Role>,
    tenant: Option<Tenant>,
}

pub struct ActorRolePayload {
    action: PermissionRelAction,
    actor_id: ArcUuid7,
    role_id: ArcUuid7,
    tenant_id: Option<TenantId>,
}
