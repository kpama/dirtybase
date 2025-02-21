use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use anyhow::anyhow;
use crypto::aead::rand_core::RngCore;
use dirtybase_helper::hash::sha256;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{auth_user_status::AuthUserStatus, generate_salt},
    db::{
        base::helper::generate_ulid,
        types::{ArcUuid7, BooleanField, IntegerField, IntoColumnAndValue, OptionalDateTimeField},
        ColumnAndValueBuilder,
    },
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use super::ParseToken;

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthUser {
    id: ArcUuid7,
    username: Arc<String>,
    email_hash: Arc<String>,
    email_verified: BooleanField,
    status: AuthUserStatus,
    reset_password: BooleanField,
    #[serde(skip_deserializing, skip_serializing)]
    password: Arc<String>,
    #[serde(skip_deserializing, skip_serializing)]
    salt: Arc<String>,
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

impl Default for AuthUser {
    fn default() -> Self {
        let username = generate_ulid();
        let mut bytes = [0u8, 32];
        crypto::common::rand_core::OsRng.fill_bytes(&mut bytes);
        let salt = SaltString::generate(&mut OsRng).to_string();
        crypto::common::rand_core::OsRng.fill_bytes(&mut bytes);
        let password = dirtybase_helper::hash::sha256::hash_bytes(&bytes);
        let email_hash = dirtybase_helper::hash::sha256::hash_str(&username);
        Self {
            id: ArcUuid7::default(),
            username: username.into(),
            email_hash: email_hash.into(),
            salt: salt.into(),
            password: password.into(),
            reset_password: true,
            email_verified: false,
            status: AuthUserStatus::Pending,
            login_attempt: 0,
            is_sys_admin: false,
            last_login_at: None,
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

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn username(&self) -> Arc<String> {
        self.username.clone()
    }

    pub fn set_username(&mut self, username: &str) {
        self.username = username.to_string().into()
    }

    pub fn email_hash(&self) -> Arc<String> {
        self.email_hash.clone()
    }

    pub fn set_email(&mut self, email: &str) {
        self.email_hash = sha256::hash_str(email).into();
    }

    pub fn reset_password(&self) -> bool {
        self.reset_password
    }

    pub fn set_reset_password(&mut self, reset: bool) {
        self.reset_password = reset;
    }

    pub fn set_password(&mut self, password: &str) -> anyhow::Result<()> {
        self.password = Self::hash_password(password)?.into();
        Ok(())
    }

    pub fn verify_password(&self, raw_password: &str) -> bool {
        Self::check_password(raw_password, &self.password)
    }

    pub fn generate_token(&self) -> String {
        ParseToken::generate_token(&self.salt, self.id())
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
        write!(f, "{}", self.id.to_string())
    }
}

#[derive(Default, Debug, serde::Deserialize)]
pub struct AuthUserPayload {
    #[serde(skip_deserializing)]
    pub id: Option<ArcUuid7>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub email_verified: Option<bool>,
    #[serde(default)]
    pub status: Option<AuthUserStatus>,
    #[serde(default)]
    pub reset_password: Option<bool>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub rotate_salt: bool,
}

impl IntoColumnAndValue for AuthUserPayload {
    fn into_column_value(&self) -> crate::db::types::ColumnAndValue {
        let mut builder = ColumnAndValueBuilder::new()
            .try_to_insert("id", self.id.as_ref())
            .try_to_insert("username", self.username.as_ref())
            .try_to_insert("email", self.email.as_ref())
            .try_to_insert("email_verified", self.email_verified)
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

        builder.build()
    }
}
