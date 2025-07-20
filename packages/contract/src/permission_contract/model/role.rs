use anyhow::Ok;
use dirtybase_helper::time::current_datetime;
use serde::{Deserialize, Serialize};

use crate::db_contract::types::{
    ArcUuid7, DateTimeField, FromColumnAndValue, LabelField, NameField, OptionalNameField,
    StringField,
};

use super::PermissionRecordAction;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Role {
    id: ArcUuid7,
    name: NameField,
    label: LabelField,
    description: StringField,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
}

impl Role {
    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn name(&self) -> NameField {
        self.name.clone()
    }

    pub fn label(&self) -> LabelField {
        self.label.clone()
    }

    pub fn description(&self) -> StringField {
        self.description.clone()
    }

    pub fn touch_created_at(&mut self) {
        self.created_at = current_datetime().into();
    }

    pub fn touch_updated_at(&mut self) {
        self.updated_at = current_datetime().into();
    }

    pub fn touch_deleted_at(&mut self) {
        self.deleted_at = current_datetime().into()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RolePayload {
    pub sate_action: Option<PermissionRecordAction>,
    pub id: Option<ArcUuid7>,
    pub name: OptionalNameField,
    pub label: Option<LabelField>,
    pub description: Option<StringField>,
}

impl FromColumnAndValue for Role {
    fn from_column_value(
        mut cv: crate::db_contract::types::ColumnAndValue,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let role = Self {
            id: cv
                .remove("id")
                .ok_or(anyhow::anyhow!("id not set for role"))?
                .into(),
            name: cv
                .remove("name")
                .ok_or(anyhow::anyhow!("name not set for role"))?
                .into(),
            label: cv
                .remove("label")
                .ok_or(anyhow::anyhow!("label not set for role"))?
                .into(),
            description: cv
                .remove("description")
                .ok_or(anyhow::anyhow!("description not set for role"))?
                .into(),
            created_at: cv.remove("created_at").map(DateTimeField::from),
            updated_at: cv.remove("updated_at").map(DateTimeField::from),
            deleted_at: cv.remove("updated_at").map(DateTimeField::from),
        };

        if !cv.is_empty() {
            return Err(anyhow::anyhow!(
                "one or more role column value was not handled: {:#?}",
                cv.keys().collect::<Vec<&String>>()
            ));
        }

        Ok(role)
    }
}
