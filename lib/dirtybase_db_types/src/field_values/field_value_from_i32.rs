use super::FieldValue;

impl From<i32> for FieldValue {
    fn from(value: i32) -> Self {
        Self::I64(value.into())
    }
}

impl FromIterator<i32> for FieldValue {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<i32>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<i32>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<i32, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<i32, E>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
