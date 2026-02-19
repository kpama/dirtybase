use crate::db::field_values::FieldValue;

impl From<FieldValue> for i32 {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::I32(v) => v,
            _ => 0,
        }
    }
}

impl From<&FieldValue> for i32 {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::I32(v) => *v,
            _ => 0,
        }
    }
}

impl From<FieldValue> for Option<i32> {
    fn from(value: FieldValue) -> Self {
        if FieldValue::NotSet == value {
            None
        } else {
            Some(value.into())
        }
    }
}
