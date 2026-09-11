use crate::db::field_values::FieldValue;

fn plain_json_value(value: serde_json::Value) -> serde_json::Value {
    if let Ok(field_value) = serde_json::from_value::<FieldValue>(value.clone()) {
        return field_value.into();
    }

    match value {
        serde_json::Value::Object(value) => serde_json::Value::Object(
            value
                .into_iter()
                .map(|(key, value)| (key, plain_json_value(value)))
                .collect(),
        ),
        serde_json::Value::Array(value) => {
            serde_json::Value::Array(value.into_iter().map(plain_json_value).collect())
        }
        value => value,
    }
}

impl From<FieldValue> for serde_json::Value {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Null | FieldValue::NotSet => serde_json::Value::Null,
            FieldValue::I8(value) => value.into(),
            FieldValue::I16(value) => value.into(),
            FieldValue::I32(value) => value.into(),
            FieldValue::U32(value) => value.into(),
            FieldValue::U64(value) => value.into(),
            FieldValue::I64(value) => value.into(),
            FieldValue::F64(value) => serde_json::Number::from_f64(value)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            FieldValue::String(value) => value.into(),
            FieldValue::Boolean(value) => value.into(),
            FieldValue::Object(value) => serde_json::Value::Object(
                value
                    .into_iter()
                    .map(|(key, value)| (key, value.into()))
                    .collect(),
            ),
            FieldValue::Array(value) => {
                serde_json::Value::Array(value.into_iter().map(Into::into).collect())
            }
            FieldValue::Binary(value) => serde_json::to_value(value).unwrap_or_default(),
            FieldValue::Uuid(value) => value.to_string().into(),
            FieldValue::DateTime(value) | FieldValue::Timestamp(value) => {
                serde_json::to_value(value).unwrap_or_default()
            }
            FieldValue::Date(value) => serde_json::to_value(value).unwrap_or_default(),
            FieldValue::Time(value) => serde_json::to_value(value).unwrap_or_default(),
            FieldValue::Failable { field, error } => {
                if error.is_some() {
                    serde_json::Value::Null
                } else {
                    (*field).into()
                }
            }
        }
    }
}

impl From<&FieldValue> for serde_json::Value {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
    }
}

impl From<FieldValue> for serde_json::Map<String, serde_json::Value> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(content) => {
                if let Ok(serde_json::Value::Object(obj)) = serde_json::from_str(&content) {
                    obj.into_iter()
                        .map(|(key, value)| (key, plain_json_value(value)))
                        .collect()
                } else {
                    serde_json::Map::new()
                }
            }
            FieldValue::Binary(b) => {
                if let Ok(map) = serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(
                    b.as_slice(),
                ) {
                    return map;
                }

                serde_json::Map::new()
            }
            FieldValue::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (k, v) in obj {
                    map.insert(k, v.into());
                }
                map
            }
            _ => serde_json::Map::new(),
        }
    }
}

impl From<&FieldValue> for serde_json::Map<String, serde_json::Value> {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
    }
}

impl From<FieldValue> for Option<serde_json::Map<String, serde_json::Value>> {
    fn from(value: FieldValue) -> Self {
        let map: serde_json::Map<String, serde_json::Value> = value.into();

        if map.is_empty() { None } else { Some(map) }
    }
}

impl From<FieldValue> for Option<serde_json::Value> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Null | FieldValue::NotSet => None,
            _ => Some(serde_json::Value::from(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn converts_field_values_to_plain_json() {
        let value = FieldValue::Object(HashMap::from([
            (
                "actor".to_owned(),
                FieldValue::Object(HashMap::from([(
                    "id".to_owned(),
                    FieldValue::String("actor-id".to_owned()),
                )])),
            ),
            (
                "roles".to_owned(),
                FieldValue::Array(vec![FieldValue::String("admin".to_owned())]),
            ),
        ]));

        let actual: serde_json::Value = value.into();

        assert_eq!(
            actual,
            json!({"actor": {"id": "actor-id"}, "roles": ["admin"]})
        );
    }

    #[test]
    fn unwraps_tagged_json_loaded_from_a_database() {
        let encoded = json!({
            "_auth": {
                "t": "Object",
                "v": {
                    "actor": {"t": "String", "v": "actor-id"},
                    "roles": {"t": "Array", "v": [{"t": "String", "v": "admin"}]}
                }
            }
        });
        let value = FieldValue::String(encoded.to_string());

        let actual: serde_json::Map<String, serde_json::Value> = value.into();

        assert_eq!(
            actual,
            json!({"_auth": {"actor": "actor-id", "roles": ["admin"]}})
                .as_object()
                .unwrap()
                .clone()
        );
    }
}
