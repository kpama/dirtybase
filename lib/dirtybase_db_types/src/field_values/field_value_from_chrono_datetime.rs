use chrono::{DateTime, Utc};

use super::FieldValue;

impl From<DateTime<Utc>> for FieldValue {
    fn from(value: DateTime<Utc>) -> Self {
        Self::DateTime(value)
    }
}
