use crate::field_values::FieldValue;

impl From<FieldValue> for f64 {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::F64(v) => v,
            _ => 0.0_f64,
        }
    }
}

impl From<&FieldValue> for f64 {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::F64(v) => *v,
            _ => 0.0_f64,
        }
    }
}

impl From<FieldValue> for Option<f64> {
    fn from(value: FieldValue) -> Self {
        if FieldValue::NotSet == value {
            None
        } else {
            Some(value.into())
        }
    }
}
