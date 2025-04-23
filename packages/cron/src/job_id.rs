use std::{fmt::Display, sync::Arc};

use anyhow::anyhow;
use dirtybase_contract::{cli_contract::clap::parser::MatchesError, db_contract::base::helper};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct JobId(Arc<String>); // job id must be in the format "namespace::name"

impl Default for JobId {
    fn default() -> Self {
        Self(Arc::new(format!("auto::{}", helper::generate_ulid())))
    }
}

impl JobId {
    pub fn new(id: &str) -> Self {
        Self(Arc::new(id.replace(" ", "").to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate(inner: &str) -> Result<Self, anyhow::Error> {
        if !inner.contains("::") {
            return Err(anyhow!("Cron job ID must be in the format namespace::name"));
        }

        Ok(Self::new(inner))
    }
}

impl From<String> for JobId {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

impl From<&str> for JobId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl TryFrom<Option<&String>> for JobId {
    type Error = anyhow::Error;

    fn try_from(value: Option<&String>) -> Result<Self, Self::Error> {
        if let Some(id) = value {
            return Self::validate(id);
        }
        Err(anyhow!("ID string is empty."))
    }
}

impl TryFrom<Result<Option<&String>, MatchesError>> for JobId {
    type Error = anyhow::Error;
    fn try_from(value: Result<Option<&String>, MatchesError>) -> Result<Self, Self::Error> {
        if let Ok(inner) = value {
            return Self::try_from(inner);
        }
        Err(anyhow!("ID string is empty"))
    }
}

impl Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
