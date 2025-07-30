use std::sync::Arc;

use dirtybase_contract::db_contract::types::{
    ArcUuid7, DateTimeField, LabelField, NameField, StringField,
};
use dirtybase_db_macro::DirtyTable;
use dirtybase_helper::time::current_datetime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, DirtyTable)]
pub struct Permission {
    id: ArcUuid7,
    name: NameField,
    label: LabelField,
    description: StringField,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
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
