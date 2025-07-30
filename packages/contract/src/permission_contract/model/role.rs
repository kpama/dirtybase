use serde::{Deserialize, Serialize};

use crate::{
    db_contract::types::{ArcUuid7, LabelField, OptionalNameField, StringField},
    prelude::model::PermissionRecordAction,
};

pub trait RoleTrait {}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RolePayload {
    pub sate_action: Option<PermissionRecordAction>,
    pub id: Option<ArcUuid7>,
    pub name: OptionalNameField,
    pub label: Option<LabelField>,
    pub description: Option<StringField>,
}

pub struct RolePermissionPayload {
    pub rel_action: PermissionRecordAction,
    pub subject: PermissionSubject,
    pub permission_id: ArcUuid7,
}

#[derive(Debug, Clone)]
pub enum PermissionSubject {
    Role(ArcUuid7),
    Actor(ArcUuid7),
}
