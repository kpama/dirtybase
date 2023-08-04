use super::FieldValue;

impl From<u32> for FieldValue {
    fn from(value: u32) -> Self {
        Self::U64(value.into())
    }
}

impl FromIterator<u32> for FieldValue {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<u32>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<u32>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<u32, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<u32, E>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
