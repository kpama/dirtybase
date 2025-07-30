use serde::{Deserialize, Serialize};

use crate::{
    db_contract::types::ArcUuid7,
    multitenant_contract::TenantId,
    prelude::model::{PermissionRecordAction, PermissionRelAction},
};

pub trait ActorTrait {}

pub struct ActorPayload {
    pub action: Option<PermissionRecordAction>,
    pub id: Option<ArcUuid7>,
    pub user_id: Option<ArcUuid7>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorRolePayload {
    pub rel_action: PermissionRelAction,
    pub actor_id: ArcUuid7,
    pub role_id: ArcUuid7,
    pub tenant_id: Option<TenantId>,
}
