use chrono::{TimeDelta, Utc};
use dirtybase_db_macro::DirtyTable;
use dirtybase_helper::random::random_string;

use crate::db_contract::types::{BooleanField, StringField};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, DirtyTable)]
#[dirty(table = "lock_data", id = "key", no_soft_delete, no_timestamp)]
pub struct LockData {
    key: StringField,
    owner: StringField,
    pub(crate) acquired: BooleanField,
    pub(crate) blocking: BooleanField,
    data: Option<StringField>,
    expires: i64,
}

impl Default for LockData {
    fn default() -> Self {
        Self::new(random_string(16), 300)
    }
}

impl LockData {
    pub fn new<K: ToString>(key: K, expires: i64) -> Self {
        let key_str = key.to_string();
        Self {
            expires: Self::build_ts(expires),
            owner: Self::generate_owner(&key_str),
            key: key_str.into(),
            blocking: true,
            acquired: false,
            data: None,
        }
    }

    pub fn new_sharable<K: ToString>(key: K, expires: i64) -> Self {
        let mut lock = Self::new(key, expires);
        lock.blocking = false;

        lock
    }

    pub fn key(&self) -> StringField {
        self.key.clone()
    }

    pub fn set_key(&mut self, key: &str) -> &mut Self {
        self.key = key.to_string().into();
        self.owner = Self::generate_owner(&self.key);

        self
    }

    pub fn expires(&self) -> i64 {
        self.expires
    }

    pub fn set_expires(&mut self, expires: i64) -> &mut Self {
        self.expires = Self::build_ts(expires);
        self
    }

    pub fn owner(&self) -> StringField {
        self.owner.clone()
    }

    pub fn is_blocking(&self) -> BooleanField {
        self.blocking
    }

    pub fn is_acquired(&self) -> BooleanField {
        self.acquired
    }

    pub fn data(&self) -> Option<&StringField> {
        self.data.as_ref()
    }

    pub(crate) fn build_ts(expires: i64) -> i64 {
        Utc::now()
            .checked_add_signed(TimeDelta::seconds(expires))
            .unwrap_or_default()
            .timestamp()
    }

    fn generate_owner(key: &str) -> StringField {
        format!("{}||{}", key, random_string(16)).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_creation() {
        let lock = LockData::new("lock-data-test", 1);
        assert_eq!(lock.owner().is_empty(), false);
        assert_eq!(lock.expires() > Utc::now().timestamp(), true);
    }

    #[test]
    fn test_default() {
        let lock = LockData::default();
        assert_eq!(lock.owner().is_empty(), false);
        assert_eq!(lock.expires() > Utc::now().timestamp(), true);
    }

    #[test]
    fn test_owner_value() {
        let lock = LockData::default();
        assert!(
            lock.owner().contains("||"),
            "lock owner value must have the key separated by '||'"
        );

        let pieces = lock
            .owner()
            .split("||")
            .map(String::from)
            .collect::<Vec<String>>();
        assert_eq!(
            pieces.len(),
            2,
            "lock owner value when split show be in two pieces"
        );

        assert_eq!(
            pieces[0],
            lock.key().as_str(),
            "lock key must be part of the owner value"
        );
    }
}
