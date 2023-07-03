use super::FieldValue;

impl From<f64> for FieldValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<Option<f64>> for FieldValue {
    fn from(value: Option<f64>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<f64, E>> for FieldValue {
    fn from(value: Result<f64, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}
impl From<Vec<f64>> for FieldValue {
    fn from(value: Vec<f64>) -> Self {
        Self::F64s(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Option<Vec<f64>>> for FieldValue {
    fn from(value: Option<Vec<f64>>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl<E> From<Result<Vec<f64>, E>> for FieldValue {
    fn from(value: Result<Vec<f64>, E>) -> Self {
        if let Ok(v) = value {
            v.into()
        } else {
            Self::NotSet
        }
    }
}

impl FromIterator<f64> for FieldValue {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        Self::F64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<f64>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<f64>>>(iter: T) -> Self {
        Self::F64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<f64, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<f64, E>>>(iter: T) -> Self {
        Self::F64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
