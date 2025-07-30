use dirtybase_contract::{
    db_contract::types::{ArcUuid7, DateTimeField, LabelField, NameField, StringField},
    serde::{Deserialize, Serialize},
};
use dirtybase_db_macro::DirtyTable;
use dirtybase_helper::time::current_datetime;

#[derive(Debug, Clone, Default, Serialize, Deserialize, DirtyTable)]
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
