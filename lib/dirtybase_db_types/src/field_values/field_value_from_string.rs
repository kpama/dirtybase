use super::FieldValue;

impl From<String> for FieldValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&String> for FieldValue {
    fn from(value: &String) -> Self {
        Self::String(value.to_owned())
    }
}

impl FromIterator<String> for FieldValue {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|f| f.into()).collect())
    }
}

impl FromIterator<Option<String>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<String>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<String, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<String, E>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
