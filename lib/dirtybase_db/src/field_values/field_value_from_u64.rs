use super::FieldValue;

impl From<u64> for FieldValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

// impl FromIterator<u64> for FieldValue {
//     fn from_iter<T: IntoIterator<Item = u64>>(iter: T) -> Self {
//         Self::Array(iter.into_iter().map(|x| x.into()).collect())
//     }
// }

// impl FromIterator<Option<u64>> for FieldValue {
//     fn from_iter<T: IntoIterator<Item = Option<u64>>>(iter: T) -> Self {
//         Self::Array(
//             iter.into_iter()
//                 .filter(|x| x.is_some())
//                 .map(|x| x.unwrap_or_default().into())
//                 .collect(),
//         )
//     }
// }

// impl<E> FromIterator<Result<u64, E>> for FieldValue {
//     fn from_iter<T: IntoIterator<Item = Result<u64, E>>>(iter: T) -> Self {
//         Self::Array(
//             iter.into_iter()
//                 .filter(|x| x.is_ok())
//                 .map(|x| x.unwrap_or_default().into())
//                 .collect(),
//         )
//     }
// }
