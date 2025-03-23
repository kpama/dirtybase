use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use anyhow::anyhow;
use crypto::aead::rand_core::RngCore;
use dirtybase_helper::{hash::sha256, time::current_datetime};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    auth::{auth_user_status::AuthUserStatus, generate_salt},
    db::{
        base::helper::generate_ulid,
        types::{
            ArcUuid7, BooleanField, FromColumnAndValue, IntegerField, IntoColumnAndValue,
            OptionalDateTimeField,
        },
        ColumnAndValueBuilder,
    },
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use super::ParseToken;

#[derive(Clone, Validate, Serialize, Deserialize)]
pub struct AuthUser {
    id: Option<ArcUuid7>,
    #[validate(length(min = 4, max = 255))]
    username: Arc<String>,
    email_hash: Arc<String>,
    status: AuthUserStatus,
    reset_password: BooleanField,
    #[serde(skip_deserializing, skip_serializing)]
    password: Arc<String>,
    #[serde(skip_deserializing, skip_serializing)]
    salt: Arc<String>,
    login_attempt: IntegerField,
    verified_at: OptionalDateTimeField,
    #[serde(skip_deserializing)]
    last_login_at: OptionalDateTimeField,
    #[serde(skip_deserializing)]
    created_at: OptionalDateTimeField,
    #[serde(skip_deserializing)]
    updated_at: OptionalDateTimeField,
    #[serde(skip_deserializing)]
    deleted_at: OptionalDateTimeField,
}

impl Default for AuthUser {
    fn default() -> Self {
        let username = generate_ulid();
        let mut password_bytes = [0u8, 32];
        let salt = SaltString::generate(&mut OsRng).to_string();
        crypto::common::rand_core::OsRng.fill_bytes(&mut password_bytes);
        let password = SaltString::encode_b64(&password_bytes).unwrap().to_string();
        let email_hash = dirtybase_helper::hash::sha256::hash_str(&username);
        Self {
            id: None,
            username: username.into(),
            email_hash: email_hash.into(),
            salt: salt.into(),
            password: password.into(),
            reset_password: true,
            status: AuthUserStatus::Pending,
            login_attempt: 0,
            last_login_at: None,
            verified_at: None,
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    }
}

impl AuthUser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(&self) -> Option<ArcUuid7> {
        self.id.clone()
    }

    pub fn username(&self) -> Arc<String> {
        self.username.clone()
    }

    pub fn username_ref(&self) -> &str {
        return self.username.as_ref();
    }

    pub fn email_hash(&self) -> Arc<String> {
        self.email_hash.clone()
    }
    pub fn email_hash_ref(&self) -> &str {
        self.email_hash.as_ref()
    }

    pub fn reset_password(&self) -> bool {
        self.reset_password
    }

    pub fn verify_password(&self, raw_password: &str) -> bool {
        Self::check_password(raw_password, &self.password)
    }

    pub fn touch_updated_at(&mut self) {
        self.updated_at = Some(current_datetime());
    }

    pub fn touch_created_at(&mut self) {
        self.created_at = Some(current_datetime());
    }
    pub fn touch_deleted_at(&mut self) {
        self.deleted_at = Some(current_datetime());
    }
    pub fn clear_deleted_at(&mut self) {
        self.deleted_at = None;
    }

    pub fn generate_token(&self) -> Option<String> {
        if self.id.is_none() {
            None
        } else {
            Some(ParseToken::generate_token(
                &self.salt,
                &self.id().as_ref().unwrap(),
            ))
        }
    }

    pub fn validate_token(&self, token: &str) -> bool {
        if let Ok(parsed_token) = ParseToken::try_from(token.to_string()) {
            parsed_token.is_valid(&self.salt)
        } else {
            false
        }
    }

    pub fn rotate_salt(&mut self) {
        self.salt = SaltString::generate(&mut OsRng).to_string().into();
    }

    pub fn update(&mut self, payload: AuthUserPayload) {
        let mut cv = payload.into_column_value();

        if let Some(v) = cv.remove("id") {
            self.id = v.into();
        }

        if let Some(v) = cv.remove("username") {
            self.username = v.into();
        }

        if let Some(v) = cv.remove("status") {
            self.status = v.into();
        }

        if let Some(v) = cv.remove("reset_password") {
            self.reset_password = v.into();
        }

        if let Some(v) = cv.remove("password") {
            self.password = v.into();
        }

        if let Some(v) = cv.remove("email_hash") {
            self.email_hash = v.into();
        }

        if let Some(v) = cv.remove("salt") {
            self.salt = v.into();
        }

        if let Some(v) = cv.remove("verified_at") {
            self.verified_at = v.into();
        }

        if let Some(v) = cv.remove("deleted_at") {
            self.deleted_at = v.into();
        }

        if !cv.is_empty() {
            panic!("not handling all of the auth payload when transforming to `auth user`");
        }
    }

    pub(crate) fn hash_password(raw_password: &str) -> anyhow::Result<String> {
        let password = sha256::hash_str(raw_password);
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => Err(anyhow!("{}", e)),
        }
    }

    pub(crate) fn check_password(raw_password: &str, password_hash: &str) -> bool {
        let password = sha256::hash_str(raw_password);
        match PasswordHash::new(&password_hash) {
            Ok(parsed_hash) => Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok(),
            Err(e) => {
                tracing::debug!("could not parse password hash: {}, {}", password_hash, e);
                false
            }
        }
    }
}

impl Debug for AuthUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Display for AuthUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = if self.id.is_some() {
            self.id.as_ref().unwrap().to_string()
        } else {
            format!("-- guest user--")
        };
        write!(f, "{}", id)
    }
}

impl FromColumnAndValue for AuthUser {
    fn from_column_value(mut cv: crate::db::types::ColumnAndValue) -> Self {
        let mut user = Self::default();

        if let Some(v) = cv.remove("id") {
            user.id = v.into();
        }

        if let Some(v) = cv.remove("username") {
            user.username = v.into();
        }

        if let Some(v) = cv.remove("email_hash") {
            user.email_hash = v.into();
        }
        if let Some(v) = cv.remove("verified_at") {
            user.verified_at = v.into();
        }

        if let Some(v) = cv.remove("status") {
            user.status = v.into();
        }

        if let Some(v) = cv.remove("reset_password") {
            user.reset_password = v.into();
        }

        if let Some(v) = cv.remove("password") {
            user.password = v.into();
        }
        if let Some(v) = cv.remove("salt") {
            user.salt = v.into();
        }

        if let Some(v) = cv.remove("login_attempt") {
            user.login_attempt = v.into();
        }

        if let Some(v) = cv.remove("last_login_at") {
            user.last_login_at = v.into();
        }

        if let Some(v) = cv.remove("created_at") {
            user.created_at = v.into();
        }
        if let Some(v) = cv.remove("updated_at") {
            user.updated_at = v.into();
        }
        if let Some(v) = cv.remove("deleted_at") {
            user.deleted_at = v.into();
        }

        // remove database specific field
        cv.remove("internal_id");

        if !cv.is_empty() {
            tracing::error!("not handling all of column value entries: {:?}", cv);
        }

        user
    }
}
#[derive(Default, Validate, Debug, serde::Deserialize)]
pub struct AuthUserPayload {
    #[serde(skip_deserializing)]
    pub id: Option<ArcUuid7>,
    #[serde(default)]
    #[validate(length(min = 4, max = 256))]
    pub username: Option<String>,
    #[serde(default)]
    #[validate(email(message = "most be a valid email address"))]
    pub email: Option<String>,
    #[serde(default)]
    pub status: Option<AuthUserStatus>,
    #[serde(default)]
    pub reset_password: Option<bool>,
    #[serde(default)]
    #[validate(length(min = 8))]
    pub password: Option<String>,
    #[serde(default)]
    pub rotate_salt: bool,
    #[serde(default)]
    pub verified_at: OptionalDateTimeField,
    #[serde(default)]
    pub soft_delete: bool,
    #[serde(default)]
    pub restore: bool,
}

impl IntoColumnAndValue for AuthUserPayload {
    fn into_column_value(&self) -> crate::db::types::ColumnAndValue {
        let mut builder = ColumnAndValueBuilder::new()
            .try_to_insert("id", self.id.as_ref())
            .try_to_insert("username", self.username.as_ref())
            .try_to_insert("verified_at", self.verified_at)
            .try_to_insert("status", self.status.as_ref())
            .try_to_insert("reset_password", self.reset_password.as_ref());

        if let Some(password) = self.password.as_ref() {
            builder = builder.add_field("password", AuthUser::hash_password(&password));
        }
        if let Some(email) = self.email.as_ref() {
            builder = builder.add_field::<String>("email_hash", sha256::hash_str(email));
        }

        if self.rotate_salt {
            builder = builder.add_field("salt", generate_salt());
        }

        if self.soft_delete {
            builder = builder.add_field("deleted_at", current_datetime());
        }

        if self.restore {
            builder = builder.add_field("deleted_at", ());
        }

        builder.build()
    }
}

impl From<AuthUserPayload> for AuthUser {
    fn from(payload: AuthUserPayload) -> Self {
        let mut user = Self::default();
        user.update(payload);
        user
    }
}
