use std::str::FromStr;

use dirtybase_helper::uuid::Uuid;

use crate::db::field_values::FieldValue;

impl From<Uuid> for FieldValue {
    fn from(value: Uuid) -> Self {
        FieldValue::Uuid(value)
    }
}

impl From<FieldValue> for Uuid {
    fn from(value: FieldValue) -> Self {
        field_value_to_uuid(value).expect("could not parse field value to UUID")
    }
}

fn field_value_to_uuid(value: FieldValue) -> Result<Uuid, String> {
    match value {
        FieldValue::Uuid(uuid) => Ok(uuid.into()),
        FieldValue::Binary(bytes) => {
            if bytes.len() == 36 {
                match String::from_utf8(bytes.clone()) {
                    Ok(st) => Ok(Uuid::from_str(&st).map_err(|e| format!("{e}"))?),
                    Err(e) => Err(format!("{e}")),
                }
            } else if bytes.len() == 16 {
                match Uuid::from_slice(&bytes) {
                    Ok(uuid) => Ok(uuid),
                    Err(e) => Err(format!("{e}")),
                }
            } else {
                Err("string is not a valid uuid7".to_string())
            }
        }
        FieldValue::String(v) => Ok(Uuid::parse_str(&v).map_err(|e| format!("{e}"))?),
        _ => {
            tracing::error!("could not parse field value to uuid");
            Err("could not parse field value to uuid".to_string())
        }
    }
}
