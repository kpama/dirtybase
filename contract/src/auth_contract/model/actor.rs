use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use anyhow::anyhow;
use crypto::aead::rand_core::RngCore;
use dirtybase_common::db::types::{DateTimeField, StringField};
use dirtybase_db_macro::DirtyTable;
use dirtybase_helper::hash::sha256;
use jsonwebtoken::{EncodingKey, Header, encode};
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
    prelude::Context,
};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use super::ParseToken;

#[derive(Clone, Validate, Serialize, DirtyTable, Deserialize)]
#[dirty(table = "auth_actors", soft_deletable, id_not_auto, timestampable)]
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
    manager: Arc<PermissionManager>,

    #[serde(skip)]
    #[dirty(skip)]
    current_role: Role,
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
            manager: Arc::default(),
            current_role: Role::create_guest(),
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
        self.manager = Arc::new(manager);
    }

    pub fn current_role(&self) -> &Role {
        &self.current_role
    }

    pub fn set_current_role(&mut self, role: Role) {
        self.current_role = role;
    }

    pub async fn has_all<T: ToString>(&self, actions: &[&str], context: &Context) -> bool {
        self.manager.all(actions, context).await
    }

    pub async fn has_any(&self, actions: &[&str], context: &Context) -> bool {
        self.manager.any(actions, context).await
    }

    pub async fn can<T: ToString>(&self, action: T, context: &Context) -> bool {
        self.manager.can(action, context).await
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

    pub fn generate_jwt_token_id(&self) -> String {
        sha256::hash_string(format!(
            "{}{}",
            match self.id() {
                Some(id) => id.to_string(),
                None => "".into(),
            },
            &self.salt
        ))
    }

    pub fn varify_jwt_token_id(&self, id: &str) -> bool {
        self.generate_jwt_token_id() == id
    }

    pub fn generate_jwt_claim(&self) -> ActorJWTClaims {
        self.into()
    }

    pub fn generate_signed_jwt(&self, key: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
        self.sign_jwt_claims(&self.generate_jwt_claim(), key)
    }

    pub fn sign_jwt_claims(
        &self,
        claims: &ActorJWTClaims,
        key: &[u8],
    ) -> Result<String, jsonwebtoken::errors::Error> {
        encode(&Header::default(), &claims, &EncodingKey::from_secret(key))
    }

    pub fn verify_jwt_claim(&self, claim: &ActorJWTClaims) -> bool {
        if let Some(id) = &claim.jti {
            return *id == self.generate_jwt_token_id();
        }

        false
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
    #[serde(default)]
    pub with_active_role: Option<ArcUuid7>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorJWTClaims {
    pub aud: Option<String>,
    pub sub: Option<String>,
    pub exp: Option<u64>,
    pub iat: Option<u64>,
    pub nbf: Option<u64>,
    pub jti: Option<String>,
    pub iss: Option<String>,
    pub private: serde_json::map::Map<String, serde_json::Value>,
}

impl Display for ActorJWTClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ActorJWTClaims {{ sub: {:?}, private: {:?} }}",
            self.sub, self.private
        )
    }
}

impl Default for ActorJWTClaims {
    fn default() -> Self {
        let exp = dirtybase_helper::time::now().add_days(30).timestamp() as u64;
        let iat = dirtybase_helper::time::now().timestamp() as u64;
        Self {
            aud: None,
            sub: None,
            exp: Some(exp),
            iat: Some(iat),
            nbf: Some(iat),
            jti: None,
            iss: None,
            private: serde_json::map::Map::new(),
        }
    }
}

impl ActorJWTClaims {
    pub fn add_registered_claim(&mut self, key: &str, value: serde_json::Value) {
        match key {
            "aud" => self.aud = value.as_str().map(|s| s.to_string()),
            "sub" => self.sub = value.as_str().map(|s| s.to_string()),
            "exp" => self.exp = value.as_u64(),
            "iat" => self.iat = value.as_u64(),
            "nbf" => self.nbf = value.as_u64(),
            "jti" => self.jti = value.as_str().map(|s| s.to_string()),
            "iss" => self.iss = value.as_str().map(|s| s.to_string()),
            _ => {
                // If the key is not a registered claim, add it to private claims
                self.private.insert(key.to_string(), value);
            }
        }
    }
    pub fn add_private_claim<T: Serialize>(&mut self, key: &str, value: T) {
        if let Ok(v) = serde_json::to_value(value) {
            self.private.insert(key.to_string(), v);
        }
    }
}

impl From<&Actor> for ActorJWTClaims {
    fn from(actor: &Actor) -> Self {
        let mut claims = ActorJWTClaims::default();
        claims.sub = actor.id().map(|id| id.to_string());
        claims.jti = actor.generate_jwt_token_id().into();
        if let Some(role_id) = actor.current_role().id() {
            claims.private.insert(
                "_ar".to_string(),
                serde_json::Value::String(role_id.to_string()),
            );
        }
        claims
    }
}
