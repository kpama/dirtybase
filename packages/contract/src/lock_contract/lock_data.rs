use chrono::{TimeDelta, Utc};
use dirtybase_helper::random::random_string;

use crate::db_contract::{
    types::{FromColumnAndValue, StringField, ToColumnAndValue},
    ColumnAndValueBuilder,
};

#[derive(Debug, Clone)]
pub struct LockData {
    key: StringField,
    owner: StringField,
    expires: i64,
}

impl FromColumnAndValue for LockData {
    fn from_column_value(
        column_and_value: crate::db_contract::types::ColumnAndValue,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let mut data = Self::default();

        data.key = if let Some(v) = column_and_value.get("key") {
            v.into()
        } else {
            return Err(anyhow::anyhow!("'key' field missing"));
        };

        data.owner = if let Some(v) = column_and_value.get("owner") {
            v.into()
        } else {
            return Err(anyhow::anyhow!("'owner' field missing"));
        };

        data.expires = if let Some(v) = column_and_value.get("expires") {
            v.into()
        } else {
            return Err(anyhow::anyhow!("'expires' field missing"));
        };

        Ok(data)
    }
}

impl ToColumnAndValue for LockData {
    fn to_column_value(&self) -> Result<crate::db_contract::types::ColumnAndValue, anyhow::Error> {
        Ok(ColumnAndValueBuilder::new()
            .add("key", self.key())
            .add("owner", self.owner())
            .add("expires", self.expires())
            .build())
    }
}

impl Default for LockData {
    fn default() -> Self {
        Self::new(random_string(16), 5)
    }
}

impl LockData {
    pub fn new<K: ToString>(key: K, expires: i64) -> Self {
        Self {
            expires: Self::build_ts(expires),
            key: key.to_string().into(),
            owner: random_string(16).into(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn set_key(&mut self, key: &str) -> &mut Self {
        self.key = key.to_string().into();
        self
    }

    pub fn expires(&self) -> i64 {
        self.expires
    }

    pub fn set_expires(&mut self, expires: i64) -> &mut Self {
        self.expires = Self::build_ts(expires);
        self
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub(crate) fn build_ts(expires: i64) -> i64 {
        Utc::now()
            .checked_add_signed(TimeDelta::seconds(expires))
            .unwrap_or_default()
            .timestamp()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_creation() {
        let lock = LockData::new("-", 1);
        assert_eq!(lock.owner().is_empty(), false);
        assert_eq!(lock.expires() > Utc::now().timestamp(), true);
    }

    #[test]
    fn test_default() {
        let lock = LockData::default();
        assert_eq!(lock.owner().is_empty(), false);
        assert_eq!(lock.expires() > Utc::now().timestamp(), true);
    }
}
