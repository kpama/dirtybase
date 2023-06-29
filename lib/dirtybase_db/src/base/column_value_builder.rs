use super::{field_values::FieldValue, types::ColumnAndValue};
use std::cell::RefCell;

pub struct ColumnAndValueBuilder {
    data: RefCell<ColumnAndValue>,
}

impl ColumnAndValueBuilder {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(ColumnAndValue::new()),
        }
    }

    /// Alias to `insert`
    pub fn add<V: Into<FieldValue>>(self, key: &str, value: V) -> Self {
        self.data.borrow_mut().insert(key.into(), value.into());
        self
    }

    pub fn insert<V: Into<FieldValue>>(self, key: &str, value: V) -> Self {
        self.data.borrow_mut().insert(key.into(), value.into());
        self
    }

    pub fn try_to_insert<V: Into<FieldValue>>(self, key: &str, value: Option<V>) -> Self {
        if let Some(value) = value {
            return self.insert(key, value);
        }
        self
    }

    /// Alias to `insert_field`
    pub fn add_field<V: Into<FieldValue>>(self, key: &str, value: FieldValue) -> Self {
        self.data.borrow_mut().insert(key.into(), value);
        self
    }

    pub fn insert_field<V: Into<FieldValue>>(self, key: &str, value: FieldValue) -> Self {
        self.data.borrow_mut().insert(key.into(), value);
        self
    }

    pub fn build(self) -> ColumnAndValue {
        self.data.into_inner()
    }
}
