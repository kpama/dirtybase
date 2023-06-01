use super::FieldValue;
use std::collections::HashMap;

pub struct InsertValueBuilder {
    values: HashMap<String, FieldValue>,
}

impl Default for InsertValueBuilder {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl InsertValueBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add<T: Into<FieldValue>>(mut self, field: &str, value: T) -> Self {
        self.values.insert(field.into(), value.into());
        self
    }

    pub fn multiple(mut self, key_value: HashMap<&str, FieldValue>) -> Self {
        for kv in key_value {
            self = self.add(kv.0, kv.1);
        }

        self
    }

    pub fn build(self) -> HashMap<String, FieldValue> {
        self.values
    }
}

impl From<InsertValueBuilder> for FieldValue {
    fn from(value: InsertValueBuilder) -> Self {
        value.values.into()
    }
}
