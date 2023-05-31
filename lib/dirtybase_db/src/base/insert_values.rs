use std::{collections::HashMap, fmt::Display};

pub struct InsertValueBuilder {
    values: HashMap<String, InsertValue>,
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
    pub fn add<T: Into<InsertValue>>(mut self, field: &str, value: T) -> Self {
        self.values.insert(field.into(), value.into());
        self
    }

    // pub fn add_ref<T: Into<InsertValue>, Copy>(mut self, field: &str, value: &T) -> Self {
    //     self.add(field, as)
    // }

    pub fn multiple(mut self, key_value: HashMap<&str, InsertValue>) -> Self {
        for kv in key_value {
            self = self.add(kv.0, kv.1);
        }

        self
    }

    pub fn build(self) -> HashMap<String, InsertValue> {
        self.values
    }
}

impl From<InsertValueBuilder> for InsertValue {
    fn from(value: InsertValueBuilder) -> Self {
        value.values.into()
    }
}

#[derive(Debug, PartialEq)]
pub enum InsertValue {
    Null,
    NotSet,
    U64(u64),
    U64s(Vec<u64>),
    I64(i64),
    I64s(Vec<i64>),
    F64(f64),
    F64s(Vec<f64>),
    String(String),
    Strings(Vec<String>),
    Boolean(bool),
    Object(HashMap<String, InsertValue>),
    Array(Vec<InsertValue>),
}

impl Display for InsertValue {
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
            Self::NotSet => write!(f, ""),
        }
    }
}

// cast from native types
impl From<i32> for InsertValue {
    fn from(value: i32) -> Self {
        Self::I64(value.into())
    }
}

impl From<Option<i32>> for InsertValue {
    fn from(value: Option<i32>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<i32, E>> for InsertValue {
    fn from(value: Result<i32, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<i32>> for InsertValue {
    fn from(value: Vec<i32>) -> Self {
        Self::I64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Option<Vec<i32>>> for InsertValue {
    fn from(value: Option<Vec<i32>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<i32>, E>> for InsertValue {
    fn from(value: Result<Vec<i32>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<i32> for InsertValue {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        Self::I64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<i32>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<i32>>>(iter: T) -> Self {
        Self::I64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<i32, E>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Result<i32, E>>>(iter: T) -> Self {
        Self::I64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}

impl From<u32> for InsertValue {
    fn from(value: u32) -> Self {
        Self::U64(value.into())
    }
}

impl From<Option<u32>> for InsertValue {
    fn from(value: Option<u32>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<u32, E>> for InsertValue {
    fn from(value: Result<u32, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<u32>> for InsertValue {
    fn from(value: Vec<u32>) -> Self {
        Self::U64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Option<Vec<u32>>> for InsertValue {
    fn from(value: Option<Vec<u32>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<u32>, E>> for InsertValue {
    fn from(value: Result<Vec<u32>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<u32> for InsertValue {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        Self::U64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<u32>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<u32>>>(iter: T) -> Self {
        Self::U64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<u32, E>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Result<u32, E>>>(iter: T) -> Self {
        Self::U64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}

impl From<f32> for InsertValue {
    fn from(value: f32) -> Self {
        Self::F64(value as f64)
    }
}

impl From<Option<f32>> for InsertValue {
    fn from(value: Option<f32>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<f32, E>> for InsertValue {
    fn from(value: Result<f32, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<f32>> for InsertValue {
    fn from(value: Vec<f32>) -> Self {
        Self::F64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Option<Vec<f32>>> for InsertValue {
    fn from(value: Option<Vec<f32>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<f32>, E>> for InsertValue {
    fn from(value: Result<Vec<f32>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<f32> for InsertValue {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self::F64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<f32>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<f32>>>(iter: T) -> Self {
        Self::F64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<f32, E>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Result<f32, E>>>(iter: T) -> Self {
        Self::F64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}

impl From<&str> for InsertValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Option<&str>> for InsertValue {
    fn from(value: Option<&str>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<&str, E>> for InsertValue {
    fn from(value: Result<&str, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<&str>> for InsertValue {
    fn from(value: Vec<&str>) -> Self {
        Self::Strings(value.into_iter().map(|x| x.to_owned()).collect())
    }
}

impl From<Option<Vec<&str>>> for InsertValue {
    fn from(value: Option<Vec<&str>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<'a> FromIterator<&'a str> for InsertValue {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::Strings(iter.into_iter().map(|x| x.to_owned()).collect())
    }
}

impl<'a> FromIterator<Option<&'a str>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<&'a str>>>(iter: T) -> Self {
        Self::Strings(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().to_owned())
                .collect(),
        )
    }
}

impl<'a, E> FromIterator<Result<&'a str, E>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Result<&'a str, E>>>(iter: T) -> Self {
        Self::Strings(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().to_owned())
                .collect(),
        )
    }
}

impl From<String> for InsertValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&String> for InsertValue {
    fn from(value: &String) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Option<String>> for InsertValue {
    fn from(value: Option<String>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Option<&String>> for InsertValue {
    fn from(value: Option<&String>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<String, E>> for InsertValue {
    fn from(value: Result<String, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<&String, E>> for InsertValue {
    fn from(value: Result<&String, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<String>> for InsertValue {
    fn from(value: Vec<String>) -> Self {
        Self::Strings(value)
    }
}

impl From<Option<Vec<String>>> for InsertValue {
    fn from(value: Option<Vec<String>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<String>, E>> for InsertValue {
    fn from(value: Result<Vec<String>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<String> for InsertValue {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::Strings(iter.into_iter().collect())
    }
}

impl FromIterator<Option<String>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<String>>>(iter: T) -> Self {
        Self::Strings(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap_or_default())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<String, E>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Result<String, E>>>(iter: T) -> Self {
        Self::Strings(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default())
                .collect(),
        )
    }
}

impl From<bool> for InsertValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<Option<bool>> for InsertValue {
    fn from(value: Option<bool>) -> Self {
        if let Some(b) = value {
            b.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<bool, E>> for InsertValue {
    fn from(value: Result<bool, E>) -> Self {
        if let Ok(b) = value {
            b.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<()> for InsertValue {
    fn from(_value: ()) -> Self {
        Self::Null
    }
}
impl From<Option<()>> for InsertValue {
    fn from(value: Option<()>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<(), E>> for InsertValue {
    fn from(value: Result<(), E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<HashMap<String, InsertValue>> for InsertValue {
    fn from(value: HashMap<String, InsertValue>) -> Self {
        let mut obj = HashMap::new();
        for entry in value {
            obj.insert(entry.0, entry.1);
        }

        Self::Object(obj)
    }
}

impl From<Option<HashMap<String, InsertValue>>> for InsertValue {
    fn from(value: Option<HashMap<String, InsertValue>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<HashMap<String, InsertValue>, E>> for InsertValue {
    fn from(value: Result<HashMap<String, InsertValue>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<HashMap<String, InsertValue>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = HashMap<String, InsertValue>>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|e| e.into()).collect::<Vec<Self>>())
    }
}

impl FromIterator<Option<HashMap<String, InsertValue>>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<HashMap<String, InsertValue>>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|e| e.is_some())
                .map(|e| e.into())
                .collect::<Vec<Self>>(),
        )
    }
}

impl<E> FromIterator<Result<HashMap<String, InsertValue>, E>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Result<HashMap<String, InsertValue>, E>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|e| e.is_ok())
                .map(|e| e.into())
                .collect::<Vec<Self>>(),
        )
    }
}
impl<'a> From<HashMap<&'a str, InsertValue>> for InsertValue {
    fn from(value: HashMap<&'a str, InsertValue>) -> Self {
        let mut obj = HashMap::new();
        for entry in value {
            obj.insert(entry.0.to_owned(), entry.1);
        }

        Self::Object(obj)
    }
}

impl<'a> From<Option<HashMap<&'a str, InsertValue>>> for InsertValue {
    fn from(value: Option<HashMap<&'a str, InsertValue>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<'a, E> From<Result<HashMap<&'a str, InsertValue>, E>> for InsertValue {
    fn from(value: Result<HashMap<&'a str, InsertValue>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<'a> FromIterator<HashMap<&'a str, InsertValue>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = HashMap<&'a str, InsertValue>>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|e| e.into()).collect::<Vec<Self>>())
    }
}

impl<'a> FromIterator<Option<HashMap<&'a str, InsertValue>>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<HashMap<&'a str, InsertValue>>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|f| f.is_some())
                .map(|e| e.into())
                .collect::<Vec<Self>>(),
        )
    }
}

impl From<i64> for InsertValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<Option<i64>> for InsertValue {
    fn from(value: Option<i64>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<i64, E>> for InsertValue {
    fn from(value: Result<i64, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<i64>> for InsertValue {
    fn from(value: Vec<i64>) -> Self {
        Self::I64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Option<Vec<i64>>> for InsertValue {
    fn from(value: Option<Vec<i64>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<i64>, E>> for InsertValue {
    fn from(value: Result<Vec<i64>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<i64> for InsertValue {
    fn from_iter<T: IntoIterator<Item = i64>>(iter: T) -> Self {
        Self::I64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<i64>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<i64>>>(iter: T) -> Self {
        Self::I64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<i64, E>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Result<i64, E>>>(iter: T) -> Self {
        Self::I64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}

impl From<u64> for InsertValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<Option<u64>> for InsertValue {
    fn from(value: Option<u64>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<u64, E>> for InsertValue {
    fn from(value: Result<u64, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<u64>> for InsertValue {
    fn from(value: Vec<u64>) -> Self {
        Self::U64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Option<Vec<u64>>> for InsertValue {
    fn from(value: Option<Vec<u64>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<u64>, E>> for InsertValue {
    fn from(value: Result<Vec<u64>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<u64> for InsertValue {
    fn from_iter<T: IntoIterator<Item = u64>>(iter: T) -> Self {
        Self::U64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<u64>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<u64>>>(iter: T) -> Self {
        Self::U64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<u64, E>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Result<u64, E>>>(iter: T) -> Self {
        Self::U64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}

impl From<f64> for InsertValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<Option<f64>> for InsertValue {
    fn from(value: Option<f64>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<f64, E>> for InsertValue {
    fn from(value: Result<f64, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<f64>> for InsertValue {
    fn from(value: Vec<f64>) -> Self {
        Self::F64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Option<Vec<f64>>> for InsertValue {
    fn from(value: Option<Vec<f64>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<f64>, E>> for InsertValue {
    fn from(value: Result<Vec<f64>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<f64> for InsertValue {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        Self::F64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<f64>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Option<f64>>>(iter: T) -> Self {
        Self::F64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<f64, E>> for InsertValue {
    fn from_iter<T: IntoIterator<Item = Result<f64, E>>>(iter: T) -> Self {
        Self::F64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
