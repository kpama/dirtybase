use chrono::{DateTime, Utc};

use super::FieldValue;

impl From<DateTime<Utc>> for FieldValue {
    fn from(value: DateTime<Utc>) -> Self {
        Self::DateTime(value)
    }
}

// impl From<Option<DateTime<Utc>>> for FieldValue {
//     fn from(value: Option<DateTime<Utc>>) -> Self {
//         if let Some(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }

// impl<E> From<Result<DateTime<Utc>, E>> for FieldValue {
//     fn from(value: Result<DateTime<Utc>, E>) -> Self {
//         if let Ok(v) = value {
//             v.into()
//         } else {
//             Self::NotSet
//         }
//     }
// }
