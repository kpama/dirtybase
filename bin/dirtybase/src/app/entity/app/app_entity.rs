use chrono::{DateTime, Utc};
use dirtybase_db::base::{
    field_values::FieldValue,
    helper::generate_ulid,
    types::{FromColumnAndValue, IntoColumnAndValue},
    ColumnAndValueBuilder,
};

use super::{
    APP_TABLE_COMPANY_ID_FIELD, APP_TABLE_CREATED_AT_FIELD, APP_TABLE_CREATOR_FIELD,
    APP_TABLE_DELETED_AT_FIELD, APP_TABLE_DESCRIPTION_FIELD, APP_TABLE_EDITOR_FIELD,
    APP_TABLE_ID_FIELD, APP_TABLE_INTERNAL_ID_FIELD, APP_TABLE_IS_SYSTEM_APP_FIELD,
    APP_TABLE_NAME_FIELD, APP_TABLE_UPDATED_AT_FIELD,
};

#[derive(Debug, Clone)]
pub struct AppEntity {
    pub internal_id: Option<u64>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_system_app: Option<bool>,
    pub company_id: Option<String>,
    pub creator_id: Option<String>,
    pub editor_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Default for AppEntity {
    fn default() -> Self {
        Self {
            internal_id: None,
            id: None,
            name: None,
            description: None,
            is_system_app: None,
            company_id: None,
            creator_id: None,
            editor_id: None,
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    }
}

impl AppEntity {
    pub fn new() -> Self {
        Self {
            id: Some(generate_ulid()),
            ..Self::default()
        }
    }
}

impl FromColumnAndValue for AppEntity {
    fn from_column_value(kv: dirtybase_db::base::types::ColumnAndValue) -> Self {
        Self {
            internal_id: FieldValue::from_ref_option_into(kv.get(APP_TABLE_INTERNAL_ID_FIELD)),
            id: FieldValue::from_ref_option_into(kv.get(APP_TABLE_ID_FIELD)),
            name: FieldValue::from_ref_option_into(kv.get(APP_TABLE_NAME_FIELD)),
            description: FieldValue::from_ref_option_into(kv.get(APP_TABLE_DESCRIPTION_FIELD)),
            is_system_app: FieldValue::from_ref_option_into(kv.get(APP_TABLE_IS_SYSTEM_APP_FIELD)),
            company_id: FieldValue::from_ref_option_into(kv.get(APP_TABLE_COMPANY_ID_FIELD)),
            creator_id: FieldValue::from_ref_option_into(kv.get(APP_TABLE_CREATOR_FIELD)),
            editor_id: FieldValue::from_ref_option_into(kv.get(APP_TABLE_EDITOR_FIELD)),
            created_at: FieldValue::from_ref_option_into(kv.get(APP_TABLE_CREATED_AT_FIELD)),
            updated_at: FieldValue::from_ref_option_into(kv.get(APP_TABLE_UPDATED_AT_FIELD)),
            deleted_at: FieldValue::from_ref_option_into(kv.get(APP_TABLE_DELETED_AT_FIELD)),
        }
    }
}

impl IntoColumnAndValue for AppEntity {
    fn into_column_value(self) -> dirtybase_db::base::types::ColumnAndValue {
        ColumnAndValueBuilder::new()
            .try_to_insert(APP_TABLE_ID_FIELD, self.id)
            .try_to_insert(APP_TABLE_NAME_FIELD, self.name)
            .try_to_insert(APP_TABLE_DESCRIPTION_FIELD, self.description)
            .try_to_insert(APP_TABLE_IS_SYSTEM_APP_FIELD, self.is_system_app)
            .try_to_insert(APP_TABLE_COMPANY_ID_FIELD, self.company_id)
            .try_to_insert(APP_TABLE_CREATOR_FIELD, self.creator_id)
            .try_to_insert(APP_TABLE_EDITOR_FIELD, self.editor_id)
            .build()
    }
}
