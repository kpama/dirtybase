use super::query::QueryBuilder;

#[derive(Debug)]
pub enum Value {
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
    SubQuery(QueryBuilder),
}

// i32
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::I64(value.into())
    }
}

impl From<Vec<i32>> for Value {
    fn from(value: Vec<i32>) -> Self {
        Self::I64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<i32> for Value {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        Self::F64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::U64(value.into())
    }
}

impl From<Vec<u32>> for Value {
    fn from(value: Vec<u32>) -> Self {
        Self::U64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<u32> for Value {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        Self::U64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::F64(value as f64)
    }
}

impl From<Vec<f32>> for Value {
    fn from(value: Vec<f32>) -> Self {
        Self::F64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<f32> for Value {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self::F64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<&str>> for Value {
    fn from(value: Vec<&str>) -> Self {
        Self::Strings(value.into_iter().map(|x| x.to_owned()).collect())
    }
}

impl<'a> FromIterator<&'a str> for Value {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::Strings(iter.into_iter().map(|x| x.to_owned()).collect())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Vec<String>> for Value {
    fn from(value: Vec<String>) -> Self {
        Self::Strings(value)
    }
}

impl FromIterator<String> for Value {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::Strings(iter.into_iter().collect())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<()> for Value {
    fn from(_value: ()) -> Self {
        Self::Null
    }
}

impl Value {
    pub fn to_param(&self, params: &mut Vec<String>) {
        match self {
            Self::Null => (),
            Self::U64(v) => params.push(v.to_string()),
            Self::I64(v) => params.push(v.to_string()),
            Self::F64(v) => params.push(v.to_string()),
            Self::String(v) => params.push(v.to_string()),
            Self::Boolean(v) => {
                params.push(if *v { 1.to_string() } else { 0.to_string() });
            }
            Self::U64s(v) => params.push(format!(
                "({})",
                v.as_slice()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            )),
            Self::I64s(v) => params.extend(
                v.as_slice()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            ),
            Self::F64s(v) => params.extend(
                v.as_slice()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            ),
            Self::Strings(v) => {
                let s = v.iter().map(|x| x.to_string()).collect::<Vec<String>>();
                params.extend(s);
            }
            Self::SubQuery(_) => {
                // Do not append. The specific database driver may handle this differently
            }
        }
    }
}
