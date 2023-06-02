use crate::base::field_values::FieldValue;

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
            FieldValue::U64(v) => v.clone().into(),
            _ => 0,
        }
    }
}

impl From<FieldValue> for Option<u64> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::U64(v) => Some(v),
            _ => None,
        }
    }
}

impl From<FieldValue> for Vec<u64> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::U64s(v) => v,
            _ => Vec::new(),
        }
    }
}
