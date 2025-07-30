use serde::{Deserialize, Serialize};

use crate::{
    db_contract::types::{ArcUuid7, LabelField, OptionalNameField},
    prelude::model::PermissionRecordAction,
};

pub trait PermissionTrait {}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PermissionPayload {
    pub state_action: Option<PermissionRecordAction>,
    pub id: Option<ArcUuid7>,
    pub name: OptionalNameField,
    pub label: Option<LabelField>,
    pub description: Option<LabelField>,
}
