use crate::db::field_values::FieldValue;

impl From<FieldValue> for u64 {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::U64(v) => v,
            _ => 0,
        }
    }
}

impl From<&FieldValue> for u64 {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::U64(v) => *v,
            _ => 0,
        }
    }
}

impl From<FieldValue> for Option<u64> {
    fn from(value: FieldValue) -> Self {
        if FieldValue::NotSet == value {
            None
        } else {
            Some(value.into())
        }
    }
}
