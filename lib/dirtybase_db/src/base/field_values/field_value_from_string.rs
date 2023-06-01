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

impl From<Option<String>> for FieldValue {
    fn from(value: Option<String>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Option<&String>> for FieldValue {
    fn from(value: Option<&String>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<String, E>> for FieldValue {
    fn from(value: Result<String, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<&String, E>> for FieldValue {
    fn from(value: Result<&String, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<String>> for FieldValue {
    fn from(value: Vec<String>) -> Self {
        Self::Strings(value)
    }
}

impl From<Option<Vec<String>>> for FieldValue {
    fn from(value: Option<Vec<String>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<String>, E>> for FieldValue {
    fn from(value: Result<Vec<String>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<String> for FieldValue {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::Strings(iter.into_iter().collect())
    }
}

impl FromIterator<Option<String>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<String>>>(iter: T) -> Self {
        Self::Strings(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap_or_default())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<String, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<String, E>>>(iter: T) -> Self {
        Self::Strings(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default())
                .collect(),
        )
    }
}
