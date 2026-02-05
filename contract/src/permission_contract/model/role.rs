use dirtybase_common::db::types::{ArcUuid7, DateTimeField, LabelField, NameField, StringField};
use serde::{Deserialize, Serialize};

use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Clone, Default, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "perm_roles")]
pub struct Role {
    id: Option<ArcUuid7>,
    name: NameField,
    label: LabelField,
    description: StringField,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
}

impl Role {
    pub fn id(&self) -> Option<&ArcUuid7> {
        self.id.as_ref()
    }

    pub fn set_id(&mut self, id: ArcUuid7) -> &mut Self {
        self.id = Some(id);
        self
    }

    pub fn name(&self) -> &NameField {
        &self.name
    }

    pub fn set_name(&mut self, name: NameField) -> &mut Self {
        self.name = name;
        self
    }

    pub fn label(&self) -> &LabelField {
        &self.label
    }

    pub fn set_label(&mut self, label: LabelField) -> &mut Self {
        self.label = label;
        self
    }

    pub fn description(&self) -> &StringField {
        &self.description
    }

    pub fn set_description(&mut self, description: &str) -> &mut Self {
        self.description = description.to_string().into();
        self
    }

    pub fn created_at(&self) -> Option<&DateTimeField> {
        self.created_at.as_ref()
    }

    pub fn update_at(&self) -> Option<&DateTimeField> {
        self.updated_at.as_ref()
    }

    pub fn deleted_at(&self) -> Option<&DateTimeField> {
        self.deleted_at.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersistRolePayload {
    Save { role: Role },
    Delete { id: ArcUuid7 },
    Restore { id: ArcUuid7 },
    Destroy { id: ArcUuid7 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRoleOption {
    pub check_trashed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchRolePayload {
    ById { id: ArcUuid7 },
    ByName { name: NameField },
}
