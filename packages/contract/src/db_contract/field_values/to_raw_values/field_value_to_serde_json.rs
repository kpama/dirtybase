use std::collections::HashMap;

use crate::db_contract::field_values::FieldValue;

impl From<FieldValue> for serde_json::Value {
    fn from(value: FieldValue) -> Self {
        if let Ok(r) = serde_json::to_value(value) {
            r
        } else {
            serde_json::Value::Null
        }
    }
}

impl From<&FieldValue> for serde_json::Value {
    fn from(value: &FieldValue) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            v
        } else {
            serde_json::Value::Null
        }
    }
}

impl From<FieldValue> for serde_json::Map<String, serde_json::Value> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(content) => {
                if let Ok(serde_json::Value::Object(obj)) = serde_json::from_str(&content) {
                    obj
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

impl From<FieldValue> for HashMap<String, String> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(content) => {
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    map
                } else {
                    HashMap::new()
                }
            }
            FieldValue::Object(obj) => {
                let mut map = HashMap::<String, String>::new();
                for (k, v) in obj {
                    map.insert(k, v.into());
                }
                map
            }
            _ => HashMap::new(),
        }
    }
}

impl From<&FieldValue> for HashMap<String, String> {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
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

        if map.is_empty() {
            None
        } else {
            Some(map)
        }
    }
}

impl From<FieldValue> for Option<HashMap<String, String>> {
    fn from(value: FieldValue) -> Self {
        let map: HashMap<String, String> = value.into();

        if map.is_empty() {
            None
        } else {
            Some(map)
        }
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
