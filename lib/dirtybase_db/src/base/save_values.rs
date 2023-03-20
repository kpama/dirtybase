use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
pub enum SaveValue {
    Null,
    U64(u64),
    U64s(Vec<u64>),
    I64(i64),
    I64s(Vec<i64>),
    F64(f64),
    F64s(Vec<f64>),
    String(String),
    Strings(Vec<String>),
    Boolean(bool),
    Object(HashMap<String, SaveValue>),
    Array(Vec<SaveValue>),
}

impl Display for SaveValue {
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
        }
    }
}

// cast from native types
impl From<i32> for SaveValue {
    fn from(value: i32) -> Self {
        Self::I64(value.into())
    }
}

impl From<Vec<i32>> for SaveValue {
    fn from(value: Vec<i32>) -> Self {
        Self::I64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<i32> for SaveValue {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        Self::F64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl From<u32> for SaveValue {
    fn from(value: u32) -> Self {
        Self::U64(value.into())
    }
}

impl From<Vec<u32>> for SaveValue {
    fn from(value: Vec<u32>) -> Self {
        Self::U64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<u32> for SaveValue {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        Self::U64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl From<f32> for SaveValue {
    fn from(value: f32) -> Self {
        Self::F64(value as f64)
    }
}

impl From<Vec<f32>> for SaveValue {
    fn from(value: Vec<f32>) -> Self {
        Self::F64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<f32> for SaveValue {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self::F64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl From<&str> for SaveValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<&str>> for SaveValue {
    fn from(value: Vec<&str>) -> Self {
        Self::Strings(value.into_iter().map(|x| x.to_owned()).collect())
    }
}

impl<'a> FromIterator<&'a str> for SaveValue {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::Strings(iter.into_iter().map(|x| x.to_owned()).collect())
    }
}

impl From<String> for SaveValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Vec<String>> for SaveValue {
    fn from(value: Vec<String>) -> Self {
        Self::Strings(value)
    }
}

impl FromIterator<String> for SaveValue {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::Strings(iter.into_iter().collect())
    }
}

impl From<bool> for SaveValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<()> for SaveValue {
    fn from(_value: ()) -> Self {
        Self::Null
    }
}

impl From<HashMap<String, SaveValue>> for SaveValue {
    fn from(value: HashMap<String, SaveValue>) -> Self {
        let mut obj = HashMap::new();
        for entry in value {
            obj.insert(entry.0, entry.1);
        }

        Self::Object(obj)
    }
}

impl FromIterator<HashMap<String, SaveValue>> for SaveValue {
    fn from_iter<T: IntoIterator<Item = HashMap<String, SaveValue>>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|e| e.into()).collect::<Vec<Self>>())
    }
}

impl<'a> From<HashMap<&'a str, SaveValue>> for SaveValue {
    fn from(value: HashMap<&'a str, SaveValue>) -> Self {
        let mut obj = HashMap::new();
        for entry in value {
            obj.insert(entry.0.to_owned(), entry.1);
        }

        Self::Object(obj)
    }
}

impl<'a> FromIterator<HashMap<&'a str, SaveValue>> for SaveValue {
    fn from_iter<T: IntoIterator<Item = HashMap<&'a str, SaveValue>>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|e| e.into()).collect::<Vec<Self>>())
    }
}
