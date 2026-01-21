use std::{fmt::Display, ops::Deref, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::db::field_values::FieldValue;

/// NameField
///
/// Takes a string, removes all white spaces, join the words with a dash and convert it to lowercase
/// Example: "Foo Bar" => "foo-bar"
#[derive(Debug, Default, PartialEq, Hash, Eq, Clone, Serialize, Deserialize)]
pub struct NameField(Arc<String>);

impl Validate for NameField {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let s = self.0.trim();
        if s.is_empty() || s.len() > 255 {
            let mut error = validator::ValidationErrors::new();
            let mut msg = ValidationError::new("length");
            msg = msg.with_message("Length must be between 1 and 255".into());
            error.add("inner", msg);
            return Err(error);
        }

        Ok(())
    }
}

impl Deref for NameField {
    type Target = Arc<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for NameField {
    fn from(inner: String) -> Self {
        Self::new(&inner)
    }
}

impl From<&str> for NameField {
    fn from(inner: &str) -> Self {
        Self::new(inner)
    }
}

impl FromStr for NameField {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl Display for NameField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for NameField {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<FieldValue> for NameField {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(inner) => Self(inner.into()),
            _ => Self::default(),
        }
    }
}

impl From<NameField> for FieldValue {
    fn from(value: NameField) -> Self {
        Self::String(value.0.to_string())
    }
}

impl NameField {
    pub fn new(name: &str) -> Self {
        let name = name
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("-")
            .to_string()
            .to_lowercase();
        Self(Arc::new(String::from_iter(name.chars().take(255))))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name_field1() {
        let name = NameField::new("The Quick Brown");

        assert_eq!(name.as_str(), "the-quick-brown");
    }

    #[test]
    fn test_name_field2() {
        let name = NameField::new("The   Quick Brown");

        assert_eq!(name.as_str(), "the-quick-brown");
    }

    #[test]
    pub fn test_name_field_from_str() {
        let name: NameField = "The \r \n quick Brown".into();
        assert_eq!(name.as_str(), "the-quick-brown");
    }

    #[test]
    pub fn test_name_field_to_json() {
        let name = NameField::new("The Quick Brown");

        assert_eq!(
            &serde_json::to_string(&name).unwrap(),
            "\"the-quick-brown\""
        );
        assert!(serde_json::from_str::<NameField>("\"the-quick-brown\"").is_ok());
    }
}
