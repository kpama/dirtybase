use crate::field_values::FieldValue;

impl From<FieldValue> for i64 {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::I64(v) => v,
            _ => 0,
        }
    }
}

impl From<&FieldValue> for i64 {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::I64(v) => v.clone().into(),
            _ => 0,
        }
    }
}

impl From<FieldValue> for Option<i64> {
    fn from(value: FieldValue) -> Self {
        if FieldValue::NotSet == value {
            None
        } else {
            Some(value.into())
        }
    }
}
