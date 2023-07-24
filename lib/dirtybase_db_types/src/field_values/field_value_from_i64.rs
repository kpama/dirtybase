use super::FieldValue;

impl From<i64> for FieldValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

// impl From<Option<i64>> for FieldValue {
//     fn from(value: Option<i64>) -> Self {
//         if let Some(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }

// impl<E> From<Result<i64, E>> for FieldValue {
//     fn from(value: Result<i64, E>) -> Self {
//         if let Ok(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }

impl From<Vec<i64>> for FieldValue {
    fn from(value: Vec<i64>) -> Self {
        Self::I64s(value.into_iter().map(|x| x.into()).collect())
    }
}

// impl From<Option<Vec<i64>>> for FieldValue {
//     fn from(value: Option<Vec<i64>>) -> Self {
//         if let Some(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }

// impl<E> From<Result<Vec<i64>, E>> for FieldValue {
//     fn from(value: Result<Vec<i64>, E>) -> Self {
//         if let Ok(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }

impl FromIterator<i64> for FieldValue {
    fn from_iter<T: IntoIterator<Item = i64>>(iter: T) -> Self {
        Self::I64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<i64>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<i64>>>(iter: T) -> Self {
        Self::I64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<i64, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<i64, E>>>(iter: T) -> Self {
        Self::I64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
