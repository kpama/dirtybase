use super::FieldValue;

impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Option<&str>> for FieldValue {
    fn from(value: Option<&str>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<&str, E>> for FieldValue {
    fn from(value: Result<&str, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl From<Vec<&str>> for FieldValue {
    fn from(value: Vec<&str>) -> Self {
        Self::Strings(value.into_iter().map(|x| x.to_owned()).collect())
    }
}

impl From<Option<Vec<&str>>> for FieldValue {
    fn from(value: Option<Vec<&str>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<'a> FromIterator<&'a str> for FieldValue {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::Strings(iter.into_iter().map(|x| x.to_owned()).collect())
    }
}

impl<'a> FromIterator<Option<&'a str>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<&'a str>>>(iter: T) -> Self {
        Self::Strings(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().to_owned())
                .collect(),
        )
    }
}

impl<'a, E> FromIterator<Result<&'a str, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<&'a str, E>>>(iter: T) -> Self {
        Self::Strings(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().to_owned())
                .collect(),
        )
    }
}
