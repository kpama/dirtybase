use crate::db_contract::field_values::FieldValue;

impl From<FieldValue> for Vec<u8> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Binary(data) => data,
            _ => vec![],
        }
    }
}

impl From<&FieldValue> for Vec<u8> {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Binary(data) => data.clone(),
            _ => vec![],
        }
    }
}

impl From<FieldValue> for Option<Vec<u8>> {
    fn from(value: FieldValue) -> Self {
        if FieldValue::NotSet == value {
            return None;
        }
        match value {
            FieldValue::Binary(data) if !data.is_empty() => Some(data),
            _ => None,
        }
    }
}
