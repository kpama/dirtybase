use crate::field_values::FieldValue;

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
        if FieldValue::NotSet == value {
            None
        } else {
            Some(value.into())
        }
    }
}
