use std::sync::Arc;

use dirtybase_helper::time::current_datetime;
use serde::{Deserialize, Serialize};

use crate::{
    db_contract::types::{
        ArcUuid7, DateTimeField, FromColumnAndValue, LabelField, NameField, OptionalNameField,
    },
    prelude::model::PermissionRecordAction,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Permission {
    id: ArcUuid7,
    name: NameField,
    label: LabelField,
    description: Arc<String>,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PermissionPayload {
    pub state_action: Option<PermissionRecordAction>,
    pub id: Option<ArcUuid7>,
    pub name: OptionalNameField,
    pub label: Option<LabelField>,
    pub description: Option<LabelField>,
}

impl Permission {
    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn name(&self) -> NameField {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_label(&mut self, label: &str) -> &mut Self {
        self.label = label.into();
        self
    }

    pub fn set_description(&mut self, desc: &str) -> &mut Self {
        self.description = desc.to_string().into();
        self
    }

    pub fn label(&self) -> LabelField {
        self.label.clone()
    }

    pub fn description(&self) -> Arc<String> {
        self.description.clone()
    }

    pub fn created_at(&self) -> Option<DateTimeField> {
        self.created_at
    }

    pub fn updated_at(&self) -> Option<DateTimeField> {
        self.updated_at
    }

    pub fn deleted_at(&self) -> Option<DateTimeField> {
        self.deleted_at
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

impl FromColumnAndValue for Permission {
    fn from_column_value(
        mut cv: crate::db_contract::types::ColumnAndValue,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let perm = Self {
            id: cv
                .remove("id")
                .ok_or(anyhow::anyhow!("id not set for permission"))?
                .into(),
            name: cv
                .remove("name")
                .ok_or(anyhow::anyhow!("name not set for permission"))?
                .into(),
            label: cv
                .remove("label")
                .ok_or(anyhow::anyhow!("label not set for permission"))?
                .into(),
            description: cv
                .remove("description")
                .ok_or(anyhow::anyhow!("description not set for permissio"))?
                .into(),
            created_at: cv.remove("created_at").map(DateTimeField::from),
            updated_at: cv.remove("updated_at").map(DateTimeField::from),
            deleted_at: cv.remove("updated_at").map(DateTimeField::from),
        };

        if !cv.is_empty() {
            return Err(anyhow::anyhow!(
                "one or more permission column value was not handled: {:#?}",
                cv.keys().collect::<Vec<&String>>()
            ));
        }

        Ok(perm)
    }
}
