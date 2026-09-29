use std::collections::{BTreeMap, BTreeSet};

use dirtybase_db::types::ColumnAndValue;
use dirtybase_db::{field_values::FieldValue, types::ToColumnAndValue};

use crate::{
    attribute::is_nil,
    error::{Error, Errors, Result},
    relationship::LoadedRelationship,
};

/// A materialised resource instance.
///
/// Soot works with `FieldValue` bags rather than a compiled-in Rust struct per
/// resource. That is what lets a single generic engine run every action for
/// every resource, which is the whole point of Ash's design: a resource is
/// metadata, and the engine is written once against that metadata.
///
/// Values that came from a column live in `values`. Values produced by loading a
/// relationship, running a calculation or an aggregate live in `loaded`, which is
/// tracked separately so a read action knows what to return and a
/// [`crate::domain::Domain::load`] call knows what is still missing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Record {
    values: BTreeMap<String, FieldValue>,
    loaded: BTreeMap<String, LoadedRelationship>,
    calculated: BTreeMap<String, FieldValue>,
    aggregated: BTreeMap<String, FieldValue>,
}

impl Record {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pairs<K: Into<String>>(pairs: impl IntoIterator<Item = (K, FieldValue)>) -> Self {
        Self {
            values: pairs.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            ..Default::default()
        }
    }

    pub fn get(&self, name: &str) -> Option<FieldValue> {
        self.values
            .get(name)
            .or_else(|| self.calculated.get(name))
            .or_else(|| self.aggregated.get(name))
            .cloned()
    }

    /// A string value, or the empty string when absent or another type.
    pub fn get_str(&self, name: &str) -> String {
        self.get(name).map(|v| v.to_string()).unwrap_or_default()
    }

    pub fn get_i64(&self, name: &str) -> Option<i64> {
        match self.get(name)? {
            FieldValue::I64(v) => Some(v),
            FieldValue::I32(v) => Some(v as i64),
            FieldValue::U64(v) => Some(v as i64),
            FieldValue::F64(v) => Some(v as i64),
            FieldValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn get_f64(&self, name: &str) -> Option<f64> {
        match self.get(name)? {
            FieldValue::F64(v) => Some(v),
            FieldValue::I64(v) => Some(v as f64),
            FieldValue::U64(v) => Some(v as f64),
            FieldValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn get_bool(&self, name: &str) -> bool {
        match self.get(name) {
            Some(FieldValue::Boolean(v)) => v,
            Some(FieldValue::I64(v)) => v != 0,
            _ => false,
        }
    }

    pub fn set(&mut self, name: &str, value: impl Into<FieldValue>) {
        self.values.insert(name.to_string(), value.into());
    }

    pub fn set_field_value(&mut self, name: &str, value: FieldValue) {
        self.values.insert(name.to_string(), value);
    }

    pub fn remove(&mut self, name: &str) -> Option<FieldValue> {
        self.loaded.remove(name);
        self.calculated.remove(name);
        self.aggregated.remove(name);
        self.values.remove(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
            || self.calculated.contains_key(name)
            || self.aggregated.contains_key(name)
    }

    pub fn is_nil(&self, name: &str) -> bool {
        match self.get(name) {
            Some(value) => is_nil(&value),
            None => true,
        }
    }

    /// The primary key value, read from the named column.
    pub fn primary_key(&self, primary_key_column: &str) -> Option<FieldValue> {
        self.values.get(primary_key_column).cloned()
    }
}

impl Record {
    pub fn values(&self) -> &BTreeMap<String, FieldValue> {
        &self.values
    }

    pub fn loaded(&self) -> &BTreeMap<String, LoadedRelationship> {
        &self.loaded
    }

    pub fn calculated(&self) -> &BTreeMap<String, FieldValue> {
        &self.calculated
    }

    pub fn aggregated(&self) -> &BTreeMap<String, FieldValue> {
        &self.aggregated
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded.contains_key(name)
    }

    pub fn loaded_relationship(&self, name: &str) -> Option<&LoadedRelationship> {
        self.loaded.get(name)
    }

    pub fn put_loaded(&mut self, name: &str, relationship: LoadedRelationship) {
        self.loaded.insert(name.to_string(), relationship);
    }

    pub fn put_calculated(&mut self, name: &str, value: FieldValue) {
        self.calculated.insert(name.to_string(), value);
    }

    pub fn put_aggregated(&mut self, name: &str, value: FieldValue) {
        self.aggregated.insert(name.to_string(), value);
    }

    /// The record as a column map, for writing to the database.
    pub fn to_column_and_value(&self) -> ColumnAndValue {
        self.values
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Every value the record exposes, column values plus calculated and
    /// aggregated results. This is what an action returns.
    pub fn to_flat_map(&self) -> BTreeMap<String, FieldValue> {
        let mut out = self.values.clone();
        out.extend(self.calculated.clone());
        out.extend(self.aggregated.clone());
        out
    }

    /// A JSON representation, with loaded relationships expanded into nested
    /// objects. Mirrors what a JSON:API or GraphQL extension would serialise.
    pub fn to_json(&self) -> serde_json::Value {
        let mut object = serde_json::Map::new();
        for (name, value) in self.to_flat_map() {
            object.insert(name, field_value_to_json(&value));
        }
        for (name, relationship) in &self.loaded {
            object.insert(name.clone(), relationship.to_json());
        }
        serde_json::Value::Object(object)
    }

    /// Drop loaded values that were not explicitly requested, and clear
    /// calculated and aggregated values, leaving only column data.
    ///
    /// `keep` is the set of relationship names the caller asked for. A read
    /// action returns only what it loaded, so an unrequested relationship never
    /// leaks into the output.
    pub fn retain_requested(&mut self, keep: &BTreeSet<String>) {
        let drop: Vec<String> = self
            .loaded
            .keys()
            .filter(|name| !keep.contains(*name))
            .cloned()
            .collect();
        for name in drop {
            self.loaded.remove(&name);
        }
    }

    /// Replace column values with only the names in `columns`.
    pub fn select_columns(&mut self, columns: &BTreeSet<String>) {
        self.values.retain(|name, _| columns.contains(name));
    }
}

impl ToColumnAndValue for Record {
    fn to_column_value(&self) -> std::result::Result<ColumnAndValue, anyhow::Error> {
        Ok(self.to_column_and_value())
    }
}

impl From<ColumnAndValue> for Record {
    fn from(values: ColumnAndValue) -> Self {
        Self {
            values: values.into_iter().collect(),
            ..Default::default()
        }
    }
}

impl LoadedRelationship {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::ToOne(Some(record)) => record.to_json(),
            Self::ToOne(None) => serde_json::Value::Null,
            Self::ToMany(records) => {
                serde_json::Value::Array(records.iter().map(|r| r.to_json()).collect())
            }
        }
    }
}

/// Convert a field value into plain JSON, unwrapping dirtybase's tagged
/// representation so output looks like ordinary JSON.
pub fn field_value_to_json(value: &FieldValue) -> serde_json::Value {
    match value {
        FieldValue::Null => serde_json::Value::Null,
        FieldValue::NotSet => serde_json::Value::Null,
        FieldValue::Object(map) => serde_json::Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), field_value_to_json(v)))
                .collect(),
        ),
        FieldValue::Array(items) => {
            serde_json::Value::Array(items.iter().map(field_value_to_json).collect())
        }
        FieldValue::Binary(bytes) => serde_json::Value::String(format!("<{} bytes>", bytes.len())),
        FieldValue::Failable { field, error } => {
            if error.is_some() {
                serde_json::Value::Null
            } else {
                field_value_to_json(field)
            }
        }
        other => serde_json::Value::String(other.to_string()),
    }
}

/// Read a single column out of a column map, producing a framework error
/// rather than a panic when it is missing.
pub fn require_column(values: &ColumnAndValue, column: &str, what: &str) -> Result<FieldValue> {
    values.get(column).cloned().ok_or_else(|| {
        Errors::from(Error::framework(format!(
            "{what} did not return column `{column}`"
        )))
    })
}
