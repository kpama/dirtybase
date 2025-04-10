use chrono::{DateTime, NaiveDate, NaiveTime, Utc};

use crate::db_contract::field_values::FieldValue;

// Datetime and  Timestamp
impl From<FieldValue> for DateTime<Utc> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::DateTime(v) | FieldValue::Timestamp(v) => v,
            _ => Utc::now(),
        }
    }
}

impl From<FieldValue> for Option<DateTime<Utc>> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::DateTime(v) | FieldValue::Timestamp(v) => Some(v),
            _ => None,
        }
    }
}

impl From<&FieldValue> for DateTime<Utc> {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::DateTime(v) | FieldValue::Timestamp(v) => *v,
            _ => Utc::now(),
        }
    }
}

impl From<&FieldValue> for Option<DateTime<Utc>> {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::DateTime(v) | FieldValue::Timestamp(v) => Some(*v),
            _ => None,
        }
    }
}

// Date

impl From<FieldValue> for NaiveDate {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Date(v) => v,
            _ => Utc::now().date_naive(),
        }
    }
}

impl From<FieldValue> for Option<NaiveDate> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Date(v) => Some(v),
            _ => None,
        }
    }
}

impl From<&FieldValue> for NaiveDate {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Date(v) => *v,
            _ => Utc::now().date_naive(),
        }
    }
}

impl From<&FieldValue> for Option<NaiveDate> {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Date(v) => Some(*v),
            _ => None,
        }
    }
}

// Time

impl From<FieldValue> for NaiveTime {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Time(v) => v,
            _ => Utc::now().time(),
        }
    }
}

impl From<FieldValue> for Option<NaiveTime> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Time(v) => Some(v),
            _ => None,
        }
    }
}

impl From<&FieldValue> for NaiveTime {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Time(v) => *v,
            _ => Utc::now().time(),
        }
    }
}

impl From<&FieldValue> for Option<NaiveTime> {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Time(v) => Some(*v),
            _ => None,
        }
    }
}
