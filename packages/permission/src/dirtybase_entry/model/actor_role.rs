use dirtybase_contract::{
    db_contract::types::ArcUuid7,
    multitenant_contract::{TenantId, model::Tenant},
};
use serde::{Deserialize, Serialize};

use crate::dirtybase_entry::model::{actor::Actor, role::Role};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorRole {
    actor_id: ArcUuid7,
    role_id: ArcUuid7,
    tenant_id: Option<TenantId>,
    actor: Option<Actor>,
    role: Option<Role>,
    tenant: Option<Tenant>,
}
