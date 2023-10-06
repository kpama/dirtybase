use crate::field_values::FieldValue;

impl From<FieldValue> for bool {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Boolean(v) => v,
            _ => false,
        }
    }
}

impl From<&FieldValue> for bool {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Boolean(v) => *v,
            _ => false,
        }
    }
}

impl From<FieldValue> for Option<bool> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Boolean(v) => Some(v),
            _ => None,
        }
    }
}
