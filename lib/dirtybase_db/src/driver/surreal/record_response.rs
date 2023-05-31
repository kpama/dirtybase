use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize, Debug)]
struct Id {
    id: String,
    td: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    id: Option<Thing>,
    #[serde(flatten)]
    props: Option<HashMap<String, Option<Box<Record>>>>,
}

pub fn walk_response(json_value: &mut serde_json::Value) {
    if json_value.is_object() {
        for v in json_value.as_object_mut().unwrap() {
            if v.1.is_object() {
                let id = v.1.get("id");
                let td = v.1.get("tb");
                if id.is_some() && td.is_some() {
                    *v.1 = serde_json::Value::String(format!(
                        "{}:{}",
                        td.unwrap().as_str().unwrap(),
                        id.unwrap()
                            .get("String")
                            .unwrap_or(&serde_json::Value::Null)
                            .as_str()
                            .unwrap()
                    ));
                } else {
                    walk_response(v.1);
                }
            }
        }
    }
}
