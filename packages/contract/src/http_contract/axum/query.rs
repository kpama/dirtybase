use std::collections::HashMap;

pub fn query_to_kv(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .into_iter()
        .map(|entry| {
            let key;
            let mut value = String::new();
            let mut pieces = entry.split('=');

            // key
            if let Some(k) = pieces.next() {
                if k.is_empty() {
                    return None;
                }
                key = k.to_string();
            } else {
                return None;
            }

            // value
            if let Some(v) = pieces.next() {
                value = v.to_string();
            }

            Some((key, value))
        })
        .filter(|e| e.is_some())
        .map(|e| e.unwrap())
        .collect::<HashMap<String, String>>()
}
