use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use anyhow::anyhow;
use crypto::aead::rand_core::RngCore;
use dirtybase_common::db::types::{DateTimeField, StringField};
use dirtybase_db_macro::DirtyTable;
use dirtybase_helper::hash::sha256;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    auth_contract::{
        ActorRole, Permission, PermissionManager, Role, RolePermission,
        auth_user_status::AuthUserStatus, generate_salt,
    },
    db_contract::{
        ColumnAndValueBuilder,
        base::helper::generate_ulid,
        types::{ArcUuid7, BooleanField, IntegerField, OptionalDateTimeField, ToColumnAndValue},
    },
};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use super::ParseToken;

#[derive(Clone, Validate, Serialize, DirtyTable, Deserialize)]
#[dirty(table = "auth_actors", soft_deletable, timestampable)]
pub struct Actor {
    pub(crate) id: Option<ArcUuid7>,
    #[validate(length(min = 4, max = 255))]
    username: StringField,
    email_hash: StringField,
    status: AuthUserStatus,
    reset_password: BooleanField,
    #[serde(skip)]
    password: StringField,
    #[serde(skip)]
    salt: StringField,
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

    #[dirty(rel(kind = "has_many_through", pivot = ActorRole, soft_deletable))]
    #[serde(skip_deserializing)]
    roles: Vec<Role>,

    #[dirty(rel(kind = "has_many", soft_deletable))]
    actor_roles: Vec<ActorRole>,

    #[dirty(rel(kind = "has_many_through", pivot = RolePermission, soft_deletable))]
    #[serde(skip)]
    permissions: Vec<Permission>,

    #[serde(skip)]
    #[dirty(skip)]
    manager: PermissionManager,
}

impl Default for Actor {
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
            roles: Vec::default(),
            actor_roles: Vec::default(),
            permissions: Vec::default(),
            manager: PermissionManager::default(),
        }
    }
}

impl Actor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(&self) -> Option<&ArcUuid7> {
        self.id.as_ref()
    }

    pub fn username(&self) -> Arc<String> {
        self.username.clone()
    }

    pub fn username_ref(&self) -> &str {
        self.username.as_ref()
    }

    pub fn roles(&self) -> &[Role] {
        &self.roles
    }

    pub fn created_at(&self) -> Option<&DateTimeField> {
        self.created_at.as_ref()
    }

    pub fn updated_at(&self) -> Option<&DateTimeField> {
        self.updated_at.as_ref()
    }

    pub fn deleted_at(&self) -> Option<&DateTimeField> {
        self.deleted_at.as_ref()
    }

    pub fn actor_roles(&self) -> &[ActorRole] {
        &self.actor_roles
    }

    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }

    pub fn set_perm_manager(&mut self, manager: PermissionManager) {
        self.manager = manager;
    }

    pub fn has_all<T: ToString>(&self, actions: &[&str]) -> bool {
        self.manager.all(actions)
    }

    pub fn has_any(&self, actions: &[&str]) -> bool {
        self.manager.any(actions)
    }

    pub fn can<T: ToString>(&self, action: T) -> bool {
        self.manager.can(action)
    }

    pub fn status(&self) -> AuthUserStatus {
        self.status
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

    pub fn inc_login_attempt(&mut self) {
        self.login_attempt += 1;
    }

    pub fn generate_token(&self) -> Option<String> {
        if self.id.is_none() {
            tracing::debug!("cannot generate user token. ID empty");
            None
        } else {
            Some(ParseToken::generate_token(
                &self.salt,
                self.id().as_ref().unwrap(),
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

    pub fn is_guest(&self) -> bool {
        self.id.is_none()
    }

    pub fn merge(&mut self, payload: ActorPayload) {
        let Ok(mut cv) = payload.to_column_value() else {
            return;
        };

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
        match PasswordHash::new(password_hash) {
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

impl Debug for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = if self.id.is_some() {
            self.id.as_ref().unwrap().to_string()
        } else {
            "-- guest user --".to_string()
        };
        write!(f, "{id}")
    }
}

#[derive(Default, Validate, Debug, serde::Deserialize)]
pub struct ActorPayload {
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
}

impl ActorPayload {
    pub fn new() -> Self {
        Self {
            status: Some(AuthUserStatus::Pending),
            rotate_salt: true,
            ..Default::default()
        }
    }

    pub fn for_update(id: ArcUuid7) -> Self {
        Self {
            id: Some(id),
            ..Default::default()
        }
    }
}

impl ToColumnAndValue for ActorPayload {
    fn to_column_value(&self) -> Result<crate::db_contract::types::ColumnAndValue, anyhow::Error> {
        let mut builder = ColumnAndValueBuilder::new()
            .try_to_insert("id", self.id.as_ref())
            .try_to_insert("username", self.username.as_ref())
            .try_to_insert("verified_at", self.verified_at)
            .try_to_insert("status", self.status.as_ref())
            .try_to_insert("reset_password", self.reset_password.as_ref());

        if let Some(password) = self.password.as_ref() {
            builder = builder.add_field("password", Actor::hash_password(password));
        }
        if let Some(email) = self.email.as_ref() {
            builder = builder.add_field::<String>("email_hash", sha256::hash_str(email));
        }

        if self.rotate_salt {
            builder = builder.add_field("salt", generate_salt());
        }

        Ok(builder.build())
    }
}

impl From<ActorPayload> for Actor {
    fn from(payload: ActorPayload) -> Self {
        let mut user = Self::default();
        user.merge(payload);
        user
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersistActorPayload {
    Save { actor: Actor },
    Delete { id: ArcUuid7 },
    Restore { id: ArcUuid7 },
    Destroy { id: ArcUuid7 },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FetchActorOption {
    #[serde(default)]
    pub with_trashed: bool,
    #[serde(default)]
    pub with_roles: bool,
    #[serde(default)]
    pub with_actor_roles: bool,
    #[serde(default)]
    pub with_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchActorPayload {
    ById { id: ArcUuid7 },
    ByUsername { username: String },
    ByEmailHash { email_hash: String },
}

impl FetchActorPayload {
    pub fn by_id(id: ArcUuid7) -> Self {
        Self::ById { id }
    }

    pub fn by_username(username: &str) -> Self {
        Self::ByUsername {
            username: username.to_string(),
        }
    }

    pub fn by_email(email: &str) -> Self {
        let email_hash = sha256::hash_str(email);
        Self::ByEmailHash { email_hash }
    }
}
