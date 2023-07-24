use super::{DIRTYBASE_USER_TABLE_CORE_USER_FIELD, DIRTYBASE_USER_TABLE_LOGIN_ATTEMPT_FIELD};
use crate::app::entity::dirtybase_user::DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD;
use dirtybase_db::dirtybase_db_types::{
    field_values::FieldValue,
    types::{
        ColumnAndValue, DateTimeField, FromColumnAndValue, IntoColumnAndValue, SingedIntegerField,
        UlidField,
    },
    ColumnAndValueBuilder,
};

#[derive(Debug)]
pub struct DirtybaseUserEntity {
    pub core_user_id: UlidField,
    pub login_attempt: SingedIntegerField,
    pub last_login_at: DateTimeField,
}

impl Default for DirtybaseUserEntity {
    fn default() -> Self {
        Self {
            core_user_id: None,
            login_attempt: None,
            last_login_at: None,
        }
    }
}

impl FromColumnAndValue for DirtybaseUserEntity {
    fn from_column_value(column_and_value: ColumnAndValue) -> Self {
        Self {
            core_user_id: FieldValue::from_ref_option_into(
                column_and_value.get(DIRTYBASE_USER_TABLE_CORE_USER_FIELD),
            ),
            login_attempt: FieldValue::from_ref_option_into(
                column_and_value.get(DIRTYBASE_USER_TABLE_LOGIN_ATTEMPT_FIELD),
            ),
            last_login_at: FieldValue::from_ref_option_into(
                column_and_value.get(DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD),
            ),
        }
    }
}

impl IntoColumnAndValue for DirtybaseUserEntity {
    fn into_column_value(self) -> ColumnAndValue {
        ColumnAndValueBuilder::new()
            .try_to_insert(DIRTYBASE_USER_TABLE_CORE_USER_FIELD, self.core_user_id)
            .try_to_insert(DIRTYBASE_USER_TABLE_LOGIN_ATTEMPT_FIELD, self.login_attempt)
            .try_to_insert(DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD, self.last_login_at)
            .build()
    }
}
