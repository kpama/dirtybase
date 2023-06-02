use chrono::{DateTime, Utc};
use dirtybase_db::base::{
    field_values::FieldValue,
    helper::generate_ulid,
    types::{FromColumnAndValue, IntoColumnAndValue},
    ColumnAndValueBuilder,
};

pub struct CompanyEntity {
    pub internal_id: Option<u64>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub core_user_id: Option<String>,
    pub creator: Option<String>,
    pub editor: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Default for CompanyEntity {
    fn default() -> Self {
        Self {
            internal_id: None,
            id: None,
            name: None,
            description: None,
            core_user_id: None,
            creator: None,
            editor: None,
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    }
}

impl CompanyEntity {
    pub fn new() -> Self {
        Self {
            id: Some(generate_ulid()),
            ..Self::default()
        }
    }
}

impl FromColumnAndValue for CompanyEntity {
    fn from_column_value(column_and_value: dirtybase_db::base::types::ColumnAndValue) -> Self {
        Self {
            internal_id: FieldValue::from_ref_option_into(column_and_value.get("internal_id")),
            id: FieldValue::from_ref_option_into(column_and_value.get("id")),
            name: FieldValue::from_ref_option_into(column_and_value.get("name")),
            description: FieldValue::from_ref_option_into(column_and_value.get("description")),
            core_user_id: FieldValue::from_ref_option_into(column_and_value.get("core_user_id")),
            creator: FieldValue::from_ref_option_into(column_and_value.get("creator")),
            editor: FieldValue::from_ref_option_into(column_and_value.get("editor")),
            created_at: FieldValue::from_ref_option_into(column_and_value.get("created_at")),
            updated_at: FieldValue::from_ref_option_into(column_and_value.get("created_at")),
            deleted_at: FieldValue::from_ref_option_into(column_and_value.get("created_at")),
        }
    }
}

impl IntoColumnAndValue for CompanyEntity {
    fn into_column_value(self) -> dirtybase_db::base::types::ColumnAndValue {
        ColumnAndValueBuilder::new()
            .try_to_insert("id", self.id)
            .try_to_insert("name", self.name)
            .try_to_insert("description", self.description)
            .try_to_insert("core_user_id", self.core_user_id)
            .try_to_insert("creator", self.creator)
            .try_to_insert("editor", self.editor)
            .build()
    }
}
