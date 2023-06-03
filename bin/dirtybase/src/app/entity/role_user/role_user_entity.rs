use chrono::{DateTime, Utc};
use dirtybase_db::base::{
    field_values::FieldValue,
    types::{FromColumnAndValue, IntoColumnAndValue},
    ColumnAndValueBuilder,
};

use super::{
    ROLE_USER_TABLE_CORE_APP_ROLE_ID_FIELD, ROLE_USER_TABLE_CORE_USER_ID_FIELD,
    ROLE_USER_TABLE_CREATED_AT_FIELD, ROLE_USER_TABLE_CREATOR_FIELD,
    ROLE_USER_TABLE_DELETED_AT_FIELD, ROLE_USER_TABLE_EDITOR_FIELD,
    ROLE_USER_TABLE_UPDATED_AT_FIELD,
};

#[derive(Debug, Clone)]
pub struct RoleUserEntity {
    pub core_app_role_id: Option<String>,
    pub core_user_id: Option<String>,
    pub creator_id: Option<String>,
    pub editor_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Default for RoleUserEntity {
    fn default() -> Self {
        Self {
            core_app_role_id: None,
            core_user_id: None,
            creator_id: None,
            editor_id: None,
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    }
}

impl RoleUserEntity {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FromColumnAndValue for RoleUserEntity {
    // line!() nice to know....
    fn from_column_value(column_and_value: dirtybase_db::base::types::ColumnAndValue) -> Self {
        Self {
            core_app_role_id: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_USER_TABLE_CORE_APP_ROLE_ID_FIELD),
            ),
            core_user_id: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_USER_TABLE_CORE_USER_ID_FIELD),
            ),
            creator_id: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_USER_TABLE_CREATOR_FIELD),
            ),
            editor_id: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_USER_TABLE_EDITOR_FIELD),
            ),
            created_at: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_USER_TABLE_CREATED_AT_FIELD),
            ),
            updated_at: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_USER_TABLE_UPDATED_AT_FIELD),
            ),
            deleted_at: FieldValue::from_ref_option_into(
                column_and_value.get(ROLE_USER_TABLE_DELETED_AT_FIELD),
            ),
        }
    }
}

impl IntoColumnAndValue for RoleUserEntity {
    fn into_column_value(self) -> dirtybase_db::base::types::ColumnAndValue {
        ColumnAndValueBuilder::new()
            .try_to_insert(
                ROLE_USER_TABLE_CORE_APP_ROLE_ID_FIELD,
                self.core_app_role_id,
            )
            .try_to_insert(ROLE_USER_TABLE_CORE_USER_ID_FIELD, self.core_user_id)
            .try_to_insert(ROLE_USER_TABLE_CREATOR_FIELD, self.creator_id)
            .try_to_insert(ROLE_USER_TABLE_EDITOR_FIELD, self.editor_id)
            .try_to_insert(ROLE_USER_TABLE_DELETED_AT_FIELD, self.deleted_at)
            .build()
    }
}
