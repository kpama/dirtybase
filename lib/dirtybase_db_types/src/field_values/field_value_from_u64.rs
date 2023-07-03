use super::FieldValue;

impl From<u64> for FieldValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<Option<u64>> for FieldValue {
    fn from(value: Option<u64>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<u64, E>> for FieldValue {
    fn from(value: Result<u64, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<u64>> for FieldValue {
    fn from(value: Vec<u64>) -> Self {
        Self::U64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Option<Vec<u64>>> for FieldValue {
    fn from(value: Option<Vec<u64>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<u64>, E>> for FieldValue {
    fn from(value: Result<Vec<u64>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<u64> for FieldValue {
    fn from_iter<T: IntoIterator<Item = u64>>(iter: T) -> Self {
        Self::U64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<u64>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<u64>>>(iter: T) -> Self {
        Self::U64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<u64, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<u64, E>>>(iter: T) -> Self {
        Self::U64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
