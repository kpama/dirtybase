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
#[serde(tag = "type", content = "value")]
pub enum FieldValue {
    #[serde(rename(serialize = "null"))]
    Null,
    #[serde(rename(serialize = "not_set"))]
    NotSet,
    #[serde(rename(serialize = "u64"))]
    U64(u64),
    #[serde(rename(serialize = "u64_array"))]
    U64s(Vec<u64>),
    #[serde(rename(serialize = "i64"))]
    I64(i64),
    #[serde(rename(serialize = "i64_array"))]
    I64s(Vec<i64>),
    #[serde(rename(serialize = "f64"))]
    F64(f64),
    #[serde(rename(serialize = "f64_array"))]
    F64s(Vec<f64>),
    #[serde(rename(serialize = "string"))]
    String(String),
    #[serde(rename(serialize = "string_array"))]
    Strings(Vec<String>),
    #[serde(rename(serialize = "boolean"))]
    Boolean(bool),
    #[serde(rename(serialize = "object"))]
    Object(HashMap<String, FieldValue>),
    #[serde(rename(serialize = "array"))]
    Array(Vec<FieldValue>),
    #[serde(rename(serialize = "datetime"))]
    DateTime(chrono::DateTime<chrono::Utc>),
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
    pub fn from_ref_option_ref<'a>(field: Option<&'a FieldValue>) -> &'a FieldValue {
        if let Some(f) = field {
            f
        } else {
            &Self::NotSet
        }
    }

    /// Unwraps the option, clone the FieldValue and call `into` on the cloned
    pub fn from_ref_option_into<'a, T>(field: Option<&'a FieldValue>) -> T
    where
        Self: Into<T>,
    {
        return Self::from_ref_option_ref(field).clone().into();
    }

    /// Returns a the FieldValue if Some `NotSet` when None
    pub fn from_ref_option<'a>(field: Option<&'a FieldValue>) -> FieldValue {
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
            Self::U64s(v) => write!(
                f,
                "[{}]",
                v.as_slice()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::I64s(v) => write!(
                f,
                "[{}]",
                v.as_slice()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::F64s(v) => write!(
                f,
                "[{}]",
                v.as_slice()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Self::Strings(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|e| format!("\"{}\"", e))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
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
            Self::DateTime(v) => {
                write!(f, "{}", v)
            }
            Self::Date(v) => {
                write!(f, "{}", v)
            }
            Self::Time(v) => {
                write!(f, "{}", v)
            }
            Self::NotSet => write!(f, ""),
        }
    }
}

impl From<bool> for FieldValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<Option<bool>> for FieldValue {
    fn from(value: Option<bool>) -> Self {
        if let Some(b) = value {
            b.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<bool, E>> for FieldValue {
    fn from(value: Result<bool, E>) -> Self {
        if let Ok(b) = value {
            b.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<()> for FieldValue {
    fn from(_value: ()) -> Self {
        Self::Null
    }
}
impl From<Option<()>> for FieldValue {
    fn from(value: Option<()>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<(), E>> for FieldValue {
    fn from(value: Result<(), E>) -> Self {
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

impl From<Option<HashMap<String, FieldValue>>> for FieldValue {
    fn from(value: Option<HashMap<String, FieldValue>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<HashMap<String, FieldValue>, E>> for FieldValue {
    fn from(value: Result<HashMap<String, FieldValue>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
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

impl<'a> From<Option<HashMap<&'a str, FieldValue>>> for FieldValue {
    fn from(value: Option<HashMap<&'a str, FieldValue>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<'a, E> From<Result<HashMap<&'a str, FieldValue>, E>> for FieldValue {
    fn from(value: Result<HashMap<&'a str, FieldValue>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
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
