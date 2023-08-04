use super::FieldValue;

impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl<'a> FromIterator<&'a str> for FieldValue {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::Array(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl<'a> FromIterator<Option<&'a str>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<&'a str>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<'a, E> FromIterator<Result<&'a str, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<&'a str, E>>>(iter: T) -> Self {
        Self::Array(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
