use crate::base::field_values::FieldValue;

impl From<FieldValue> for String {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => v,
            _ => "".into(),
        }
    }
}

impl From<&FieldValue> for String {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::String(v) => v.clone().into(),
            _ => "".into(),
        }
    }
}

impl From<FieldValue> for Option<String> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => Some(v),
            _ => None,
        }
    }
}

impl From<FieldValue> for Vec<String> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Strings(v) => v,
            _ => Vec::new(),
        }
    }
}
