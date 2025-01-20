use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

mod field_value_from_type;
mod insert_value;

pub mod to_raw_values;
pub use insert_value::InsertValueBuilder;

use super::types::ColumnAndValue;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum FieldValue {
    Null,
    NotSet,
    U64(u64),
    I64(i64),
    F64(f64),
    String(String),
    Boolean(bool),
    Object(HashMap<String, FieldValue>),
    Array(Vec<FieldValue>),
    Binary(Vec<u8>),
    DateTime(chrono::DateTime<chrono::Utc>),
    Timestamp(chrono::DateTime<chrono::Utc>),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
}

impl Default for FieldValue {
    fn default() -> Self {
        Self::NotSet
    }
}

impl FieldValue {
    /// Returns a reference to the FieldValue if Some, `NotSet` when None
    pub fn from_ref_option_ref(field: Option<&FieldValue>) -> &FieldValue {
        if let Some(f) = field {
            f
        } else {
            &Self::NotSet
        }
    }

    /// Unwraps the option, clone the FieldValue and call `into` on the cloned
    pub fn from_ref_option_into<T>(field: Option<&FieldValue>) -> T
    where
        Self: Into<T>,
    {
        Self::from_ref_option_ref(field).clone().into()
    }

    pub fn from_ref_option_into_option<T>(field: Option<&FieldValue>) -> Option<T>
    where
        Self: Into<T>,
    {
        if field.is_some() {
            Some(Self::from_ref_option_into(field))
        } else {
            None
        }
    }

    /// Returns the FieldValue if Some and `NotSet` when None
    pub fn from_ref_option(field: Option<&FieldValue>) -> FieldValue {
        if let Some(f) = field {
            f.clone()
        } else {
            Self::NotSet
        }
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::U64(v) => write!(f, "{}", v),
            Self::I64(v) => write!(f, "{}", v),
            Self::F64(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Boolean(v) => {
                write!(f, "{}", if *v { 1 } else { 0 })
            }
            Self::Object(v) => f.write_str(&serde_json::to_string(v).unwrap()),
            Self::Binary(data) => {
                write!(f, "{}", hex::encode(data))
            }
            Self::Array(v) => {
                let mut data = "".to_owned();
                for entry in v {
                    data = format!("{} {},", data, entry);
                }

                write!(f, "[{}]", data)
            }
            Self::DateTime(v) => write!(f, "{}", v),
            Self::Timestamp(v) => write!(f, "{}", v),
            Self::Date(v) => write!(f, "{}", v),
            Self::Time(v) => write!(f, "{}", v),
            Self::NotSet => write!(f, ""),
        }
    }
}

impl From<bool> for FieldValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<()> for FieldValue {
    fn from(_value: ()) -> Self {
        Self::Null
    }
}

impl<T> From<Option<T>> for FieldValue
where
    T: Into<FieldValue>,
{
    fn from(value: Option<T>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<T> From<Vec<T>> for FieldValue
where
    T: Into<FieldValue>,
{
    fn from(value: Vec<T>) -> Self {
        Self::Array(value.into_iter().map(|i| i.into()).collect())
    }
}

impl<T> From<Box<T>> for FieldValue
where
    T: Into<FieldValue>,
{
    fn from(value: Box<T>) -> Self {
        value.into()
    }
}

impl<T, E> From<Result<T, E>> for FieldValue
where
    T: Into<FieldValue>,
{
    fn from(value: Result<T, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<HashMap<String, FieldValue>> for FieldValue {
    fn from(value: HashMap<String, FieldValue>) -> Self {
        let mut obj = HashMap::new();
        for entry in value {
            obj.insert(entry.0, entry.1);
        }

        Self::Object(obj)
    }
}

impl<'a> From<HashMap<&'a str, FieldValue>> for FieldValue {
    fn from(value: HashMap<&'a str, FieldValue>) -> Self {
        let mut obj = HashMap::new();
        for entry in value {
            obj.insert(entry.0.to_owned(), entry.1);
        }

        Self::Object(obj)
    }
}

impl<A: Into<FieldValue>> FromIterator<A> for FieldValue {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl From<&FieldValue> for ColumnAndValue {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Object(obj) => obj.clone(),
            FieldValue::String(inner) => {
                serde_json::from_str::<HashMap<String, FieldValue>>(inner).unwrap_or_default()
            }
            _ => HashMap::new(),
        }
    }
}

impl From<FieldValue> for ColumnAndValue {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Object(obj) => obj,
            FieldValue::String(inner) => {
                serde_json::from_str::<HashMap<String, FieldValue>>(&inner).unwrap_or_default()
            }
            _ => HashMap::new(),
        }
    }
}
