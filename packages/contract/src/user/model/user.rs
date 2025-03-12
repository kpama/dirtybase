use crate::{
    db::types::{ArcUuid7, BooleanField, IntegerField, OptionalDateTimeField, OptionalStringField},
    user::status::UserStatus,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::UserTrait;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct User {
    id: ArcUuid7,
    username: String,
    email: OptionalStringField,
    reset_password: BooleanField,
    status: UserStatus,
    #[serde(skip_deserializing, skip_serializing)]
    password: String,
    #[serde(skip_deserializing, skip_serializing)]
    salt: String,
    login_attempt: IntegerField,
    is_sys_admin: BooleanField,
    #[serde(skip_deserializing)]
    last_login_at: OptionalDateTimeField,
    #[serde(skip_deserializing)]
    created_at: OptionalDateTimeField,
    #[serde(skip_deserializing)]
    updated_at: OptionalDateTimeField,
    #[serde(skip_deserializing)]
    deleted_at: OptionalDateTimeField,
}

impl UserTrait for User {
    fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }
    fn username(&self) -> &String {
        &self.username
    }
    fn email(&self) -> Option<&String> {
        self.email.as_ref()
    }
    fn reset_password(&self) -> bool {
        self.reset_password
    }
    fn set_reset_password(&mut self, reset: bool) {
        self.reset_password = reset;
    }
    fn status(&self) -> UserStatus {
        self.status.clone()
    }
    fn set_status(&mut self, status: UserStatus) {
        self.status = status;
    }
    fn password(&self) -> &String {
        &self.password
    }
    fn set_password(&mut self, password: &str) {
        self.password = password.to_string();
    }
    fn salt(&self) -> &String {
        &self.salt
    }
    fn set_salt(&mut self, salt: &str) {
        self.salt = salt.to_string()
    }
    fn login_attempt(&self) -> i64 {
        self.login_attempt
    }
    fn set_login_attempt(&mut self, value: i64) {
        self.login_attempt = value;
    }
    fn is_sys_admin(&self) -> bool {
        self.is_sys_admin
    }
    fn set_is_sys_admin(&mut self, value: bool) {
        self.is_sys_admin = value;
    }
    fn last_login_at(&self) -> Option<&DateTime<Utc>> {
        self.last_login_at.as_ref()
    }
    fn set_last_login_at(&mut self, dt: DateTime<Utc>) {
        self.last_login_at = Some(dt);
    }
    fn created_at(&self) -> Option<&DateTime<Utc>> {
        self.created_at.as_ref()
    }
    fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.updated_at.as_ref()
    }
    fn deleted_at(&self) -> Option<&DateTime<Utc>> {
        self.deleted_at.as_ref()
    }
    fn set_deleted_at(&mut self, dt: DateTime<Utc>) {
        self.deleted_at = Some(dt);
    }
}
