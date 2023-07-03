use super::FieldValue;

impl From<i32> for FieldValue {
    fn from(value: i32) -> Self {
        Self::I64(value.into())
    }
}

impl From<Option<i32>> for FieldValue {
    fn from(value: Option<i32>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<i32, E>> for FieldValue {
    fn from(value: Result<i32, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<i32>> for FieldValue {
    fn from(value: Vec<i32>) -> Self {
        Self::I64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Option<Vec<i32>>> for FieldValue {
    fn from(value: Option<Vec<i32>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<i32>, E>> for FieldValue {
    fn from(value: Result<Vec<i32>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<i32> for FieldValue {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        Self::I64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<i32>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<i32>>>(iter: T) -> Self {
        Self::I64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<i32, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<i32, E>>>(iter: T) -> Self {
        Self::I64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
