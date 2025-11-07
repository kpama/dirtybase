use std::{fmt::Display, ops::Deref, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::db::field_values::FieldValue;

use super::NameField;

/// LabelField
///
/// Takes a string and make sure it is not above 255 and lower than 3
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LabelField(Arc<String>);

impl Validate for LabelField {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        if self.0.trim().is_empty() || self.0.len() > 255 {
            let mut error = validator::ValidationErrors::new();
            let mut msg = ValidationError::new("length");
            msg = msg.with_message("Length must be between 1 and 255".into());
            error.add("inner", msg);
            return Err(error);
        }

        Ok(())
    }
}

impl Deref for LabelField {
    type Target = Arc<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for LabelField {
    fn from(inner: String) -> Self {
        Self::new(&inner)
    }
}

impl From<&str> for LabelField {
    fn from(inner: &str) -> Self {
        Self::new(inner)
    }
}

impl FromStr for LabelField {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl Display for LabelField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for LabelField {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<FieldValue> for LabelField {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(inner) => Self(inner.into()),
            _ => Self::default(),
        }
    }
}

impl From<LabelField> for FieldValue {
    fn from(value: LabelField) -> Self {
        Self::String(value.0.to_string())
    }
}

impl LabelField {
    pub fn new(name: &str) -> Self {
        Self(Arc::new(String::from_iter(name.chars().take(255))))
    }
}

impl From<LabelField> for NameField {
    fn from(value: LabelField) -> Self {
        NameField::from(value.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_label_field1() {
        let name = LabelField::new("The Quick Brown");

        assert_eq!(name.as_str(), "The Quick Brown");
    }

    #[test]
    fn test_label_field2() {
        let name = LabelField::new(&"a".repeat(256));
        assert_eq!(name.len(), 255);
        assert_eq!(name.as_str(), "a".repeat(255));
    }
}
