use crate::db::field_values::FieldValue;

impl From<FieldValue> for i8 {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::I8(v) => v,
            _ => 0,
        }
    }
}

impl From<&FieldValue> for i8 {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::I8(v) => *v,
            _ => 0,
        }
    }
}

impl From<FieldValue> for Option<i8> {
    fn from(value: FieldValue) -> Self {
        if FieldValue::NotSet == value {
            None
        } else {
            Some(value.into())
        }
    }
}
