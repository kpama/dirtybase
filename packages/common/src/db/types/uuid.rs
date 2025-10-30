use dirtybase_helper::uuid::Uuid;

use crate::db::field_values::FieldValue;

impl From<Uuid> for FieldValue {
    fn from(value: Uuid) -> Self {
        FieldValue::Uuid(value)
    }
}

impl From<FieldValue> for Uuid {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Uuid(inner) => inner,
            _ => Uuid::nil(),
        }
    }
}
