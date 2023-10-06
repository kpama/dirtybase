use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

mod field_value_from_chrono_datetime;
mod field_value_from_f32;
mod field_value_from_f64;
mod field_value_from_i32;
mod field_value_from_i64;
mod field_value_from_str;
mod field_value_from_string;
mod field_value_from_u32;
mod field_value_from_u64;
mod insert_value;

pub mod to_raw_values;
pub use insert_value::InsertValueBuilder;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum FieldValue {
    #[serde(rename(serialize = "null"))]
    Null,
    #[serde(rename(serialize = "not_set"))]
    NotSet,
    #[serde(rename(serialize = "u64"))]
    U64(u64),
    #[serde(rename(serialize = "i64"))]
    I64(i64),
    #[serde(rename(serialize = "f64"))]
    F64(f64),
    #[serde(rename(serialize = "string"))]
    String(String),
    #[serde(rename(serialize = "boolean"))]
    Boolean(bool),
    #[serde(rename(serialize = "object"))]
    Object(HashMap<String, FieldValue>),
    #[serde(rename(serialize = "array"))]
    Array(Vec<FieldValue>),
    #[serde(rename(serialize = "datetime"))]
    DateTime(chrono::DateTime<chrono::Utc>),
    #[serde(rename(serialize = "timestamp"))]
    Timestamp(chrono::DateTime<chrono::Utc>),
    #[serde(rename(serialize = "date"))]
    Date(chrono::NaiveDate),
    #[serde(rename(serialize = "time"))]
    Time(chrono::NaiveTime),
}

impl Default for FieldValue {
    fn default() -> Self {
        Self::NotSet
    }
}

impl FieldValue {
    /// Returns a reference to the FieldValue if Some `NotSet` when None
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
        return Self::from_ref_option_ref(field).clone().into();
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
            Self::Object(v) => {
                let mut data = "".to_owned();
                for entry in v {
                    data = format!("{} {}: {},", data, entry.0, entry.1);
                }

                write!(f, "{{{}}}", data)
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

impl FromIterator<HashMap<String, FieldValue>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = HashMap<String, FieldValue>>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|e| e.into()).collect::<Vec<Self>>())
    }
}

impl FromIterator<Option<HashMap<String, FieldValue>>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<HashMap<String, FieldValue>>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|e| e.is_some())
                .map(|e| e.into())
                .collect::<Vec<Self>>(),
        )
    }
}

impl<E> FromIterator<Result<HashMap<String, FieldValue>, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<HashMap<String, FieldValue>, E>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|e| e.is_ok())
                .map(|e| e.into())
                .collect::<Vec<Self>>(),
        )
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

impl<'a> FromIterator<HashMap<&'a str, FieldValue>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = HashMap<&'a str, FieldValue>>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|e| e.into()).collect::<Vec<Self>>())
    }
}

impl<'a> FromIterator<Option<HashMap<&'a str, FieldValue>>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<HashMap<&'a str, FieldValue>>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|f| f.is_some())
                .map(|e| e.into())
                .collect::<Vec<Self>>(),
        )
    }
}
