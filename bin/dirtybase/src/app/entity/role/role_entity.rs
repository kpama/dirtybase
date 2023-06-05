use super::{
    ROLE_TABLE_APP_ID_FIELD, ROLE_TABLE_CREATED_AT_FIELD, ROLE_TABLE_CREATOR_FIELD,
    ROLE_TABLE_DELETED_AT_FIELD, ROLE_TABLE_EDITOR_FIELD, ROLE_TABLE_ID_FIELD,
    ROLE_TABLE_INTERNAL_ID_FIELD, ROLE_TABLE_NAME_FIELD, ROLE_TABLE_UPDATED_AT_FIELD,
};
use chrono::{DateTime, Utc};
use dirtybase_db::base::{
    field_values::FieldValue,
    helper::generate_ulid,
    types::{FromColumnAndValue, InternalIdField, IntoColumnAndValue},
    ColumnAndValueBuilder,
};

#[derive(Debug, Clone)]
pub struct RoleEntity {
    pub internal_id: InternalIdField,
    pub id: Option<String>,
    pub name: Option<String>,
    pub core_app_id: Option<String>,
    pub creator_id: Option<String>,
    pub editor_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Default for RoleEntity {
    fn default() -> Self {
        Self {
            internal_id: None,
            id: None,
            name: None,
            core_app_id: None,
            creator_id: None,
            editor_id: None,
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    }
}

impl RoleEntity {
    pub fn new() -> Self {
        Self {
            id: Some(generate_ulid()),
            ..Self::default()
        }
    }
}

impl FromColumnAndValue for RoleEntity {
    fn from_column_value(column_and_value: dirtybase_db::base::types::ColumnAndValue) -> Self {
        Self {
            internal_id: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_TABLE_INTERNAL_ID_FIELD),
            ),
            id: FieldValue::from_ref_option_into(column_and_value.get(ROLE_TABLE_ID_FIELD)),
            name: FieldValue::from_ref_option_into(column_and_value.get(ROLE_TABLE_NAME_FIELD)),
            core_app_id: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_TABLE_APP_ID_FIELD),
            ),
            creator_id: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_TABLE_CREATOR_FIELD),
            ),
            editor_id: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_TABLE_EDITOR_FIELD),
            ),
            created_at: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_TABLE_CREATED_AT_FIELD),
            ),
            updated_at: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_TABLE_UPDATED_AT_FIELD),
            ),
            deleted_at: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_TABLE_DELETED_AT_FIELD),
            ),
        }
    }
}

impl IntoColumnAndValue for RoleEntity {
    fn into_column_value(self) -> dirtybase_db::base::types::ColumnAndValue {
        ColumnAndValueBuilder::new()
            .try_to_insert(ROLE_TABLE_ID_FIELD, self.id)
            .try_to_insert(ROLE_TABLE_NAME_FIELD, self.name)
            .try_to_insert(ROLE_TABLE_APP_ID_FIELD, self.core_app_id)
            .try_to_insert(ROLE_TABLE_CREATOR_FIELD, self.creator_id)
            .try_to_insert(ROLE_TABLE_EDITOR_FIELD, self.editor_id)
            .try_to_insert(ROLE_TABLE_DELETED_AT_FIELD, self.deleted_at)
            .build()
    }
}
