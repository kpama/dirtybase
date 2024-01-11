use super::FieldValue;

impl From<f64> for FieldValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

// impl FromIterator<f64> for FieldValue {
//     fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
//         Self::Array(iter.into_iter().map(|x| x.into()).collect())
//     }
// }

// impl FromIterator<Option<f64>> for FieldValue {
//     fn from_iter<T: IntoIterator<Item = Option<f64>>>(iter: T) -> Self {
//         Self::Array(
//             iter.into_iter()
//                 .filter(|x| x.is_some())
//                 .map(|x| x.unwrap().into())
//                 .collect(),
//         )
//     }
// }

// impl<E> FromIterator<Result<f64, E>> for FieldValue {
//     fn from_iter<T: IntoIterator<Item = Result<f64, E>>>(iter: T) -> Self {
//         Self::Array(
//             iter.into_iter()
//                 .filter(|x| x.is_ok())
//                 .map(|x| x.unwrap_or_default().into())
//                 .collect(),
//         )
//     }
// }
