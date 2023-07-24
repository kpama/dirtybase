use super::FieldValue;

impl From<f32> for FieldValue {
    fn from(value: f32) -> Self {
        Self::F64(value as f64)
    }
}

// impl From<Option<f32>> for FieldValue {
//     fn from(value: Option<f32>) -> Self {
//         if let Some(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }

// impl<E> From<Result<f32, E>> for FieldValue {
//     fn from(value: Result<f32, E>) -> Self {
//         if let Ok(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }

impl From<Vec<f32>> for FieldValue {
    fn from(value: Vec<f32>) -> Self {
        Self::F64s(value.into_iter().map(|x| x.into()).collect())
    }
}

// impl From<Option<Vec<f32>>> for FieldValue {
//     fn from(value: Option<Vec<f32>>) -> Self {
//         if let Some(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }

// impl<E> From<Result<Vec<f32>, E>> for FieldValue {
//     fn from(value: Result<Vec<f32>, E>) -> Self {
//         if let Ok(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }

impl FromIterator<f32> for FieldValue {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self::F64s(iter.into_iter().map(|x| x.into()).collect())
    }
}

impl FromIterator<Option<f32>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Option<f32>>>(iter: T) -> Self {
        Self::F64s(
            iter.into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().into())
                .collect(),
        )
    }
}

impl<E> FromIterator<Result<f32, E>> for FieldValue {
    fn from_iter<T: IntoIterator<Item = Result<f32, E>>>(iter: T) -> Self {
        Self::F64s(
            iter.into_iter()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap_or_default().into())
                .collect(),
        )
    }
}
