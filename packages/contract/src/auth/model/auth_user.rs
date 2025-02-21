use std::fmt::{write, Debug, Display};

use crypto::aead::rand_core::RngCore;
use dirtybase_helper::ulid::UlidString;
use serde::{Deserialize, Serialize};

use crate::{
    auth::auth_user_status::AuthUserStatus,
    db::{
        base::helper::generate_ulid,
        types::{ArcUuid7, BooleanField, IntegerField, OptionalDateTimeField},
    },
};

#[derive(Default, Clone, Serialize, Deserialize)]
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
        let salt = dirtybase_helper::hash::sha256::hash_bytes(&bytes);
        crypto::common::rand_core::OsRng.fill_bytes(&mut bytes);
        let password = dirtybase_helper::hash::sha256::hash_bytes(&bytes);
        let email_hash = dirtybase_helper::hash::sha256::hash_str(&username);
        Self {
            id: ArcUuid7::default(),
            username,
            email_hash,
            salt,
            password,
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

impl AuthUser {}

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
