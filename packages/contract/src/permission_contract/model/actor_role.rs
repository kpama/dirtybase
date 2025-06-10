use serde::{Deserialize, Serialize};

use crate::{
    db_contract::types::{ArcUuid7, ColumnAndValue, FromColumnAndValue},
    multitenant_contract::{model::Tenant, TenantId},
};

use super::{actor::Actor, role::Role, PermissionRelAction};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ActorRole {
    actor_id: ArcUuid7,
    role_id: ArcUuid7,
    tenant_id: Option<TenantId>,
    actor: Option<Actor>,
    role: Option<Role>,
    tenant: Option<Tenant>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorRolePayload {
    pub rel_action: PermissionRelAction,
    pub actor_id: ArcUuid7,
    pub role_id: ArcUuid7,
    pub tenant_id: Option<TenantId>,
}

impl FromColumnAndValue for ActorRole {
    fn from_column_value(
        mut cv: crate::db_contract::types::ColumnAndValue,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        //

        /*
        tenant_id: Option<TenantId>,
        actor: Option<Actor>,
        role: Option<Role>,
        tenant: Option<Tenant>,
             */
        let actor_role = Self {
            actor_id: cv
                .remove("actor_id")
                .ok_or(anyhow::anyhow!("actor_id not set for actor role"))?
                .into(),
            role_id: cv
                .remove("role_id")
                .ok_or(anyhow::anyhow!("role_id not set for actor role"))?
                .into(),

            tenant_id: cv
                .remove("tenant_id")
                .map(TenantId::from)
                .ok_or(anyhow::anyhow!("tenant not set for actor role"))?
                .into(),
            actor: if let Some(actor) = cv.remove("actor") {
                Actor::from_column_value(ColumnAndValue::from(actor)).ok()
            } else {
                None
            },
            role: if let Some(role) = cv.remove("role") {
                Role::from_column_value(ColumnAndValue::from(role)).ok()
            } else {
                None
            },
            tenant: None,
        };

        if !cv.is_empty() {
            return Err(anyhow::anyhow!(
                "one or more actor role column values were not handled: {:#?}",
                cv.keys().collect::<Vec<&String>>()
            ));
        }
        Err(anyhow::anyhow!("not implemented"))
    }
}
