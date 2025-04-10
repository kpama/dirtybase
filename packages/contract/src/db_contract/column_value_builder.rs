use std::cell::RefCell;

use super::{field_values::FieldValue, types::ColumnAndValue};

pub struct ColumnAndValueBuilder {
    data: RefCell<ColumnAndValue>,
}

impl Default for ColumnAndValueBuilder {
    fn default() -> Self {
        Self::new()
    }
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
            return self.try_to_insert_field_value(key, Some(value.into()));
        }
        self
    }

    pub fn try_to_insert_field_value(self, key: &str, field: Option<FieldValue>) -> Self {
        if let Some(f) = field {
            if f != FieldValue::NotSet {
                self.data.borrow_mut().insert(key.into(), f);
            }
        }

        self
    }

    /// Alias to `insert_field`
    pub fn add_field<V: Into<FieldValue>>(self, key: &str, value: V) -> Self {
        self.data.borrow_mut().insert(key.into(), value.into());
        self
    }

    pub fn insert_field<V: Into<FieldValue>>(self, key: &str, value: FieldValue) -> Self {
        self.data.borrow_mut().insert(key.into(), value);
        self
    }

    pub fn merge(self, other: Self) -> Self {
        self.data.borrow_mut().extend(other.build());
        self
    }

    pub fn merge_column_value(self, cv: ColumnAndValue) -> Self {
        self.data.borrow_mut().extend(cv);
        self
    }

    pub fn build(self) -> ColumnAndValue {
        self.data.into_inner()
    }
}
