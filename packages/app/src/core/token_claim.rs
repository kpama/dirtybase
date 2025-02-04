use super::model::dirtybase_user::DirtybaseUserEntity;
use dirtybase_db::types::UlidField;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum JWTClaim {
    #[serde(rename(serialize = "switch_app"))]
    CanSwitchAp,
    #[serde(rename(serialize = "access_app"))]
    AccessApp,
    #[serde(rename(serialize = "reset_credential"))]
    ResetCredential,
}

impl Display for JWTClaim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccessApp => write!(f, "app_access"),
            Self::CanSwitchAp => write!(f, "switch_app"),
            Self::ResetCredential => write!(f, "reset_credential"),
        }
    }
}

#[derive(Debug, Default)]
pub struct ClaimBuilder {
    user_id: UlidField,
    salt: String,
    app: String,
    role: String,
    allow: String,
    is_sys_admin: String,
}

impl ClaimBuilder {
    pub fn new(user: &DirtybaseUserEntity) -> Self {
        Self {
            user_id: user.core_user_id.clone(),
            salt: user.salt.clone(),
            is_sys_admin: if user.is_sys_admin {
                "yes".into()
            } else {
                "no".into()
            },
            ..Self::default()
        }
    }

    pub fn user_id(mut self, id: UlidField) -> Self {
        self.user_id = id;
        self
    }

    pub fn set_salt(mut self, salt: &str) -> Self {
        self.salt = salt.into();
        self
    }

    pub fn set_app(mut self, app: &str) -> Self {
        self.app = app.into();
        self
    }

    pub fn set_role(mut self, role: &str) -> Self {
        self.role = role.into();
        self
    }

    pub fn set_allow_ref<T: ToString>(mut self, allow: &T) -> Self {
        self.allow = allow.to_string();
        self
    }
    pub fn set_allow<T: ToString>(mut self, allow: T) -> Self {
        self.allow = allow.to_string();
        self
    }

    pub fn build(self) -> HashMap<String, String> {
        let mut claims = HashMap::new();
        claims.insert("user".into(), self.user_id.to_string());
        claims.insert("salt".into(), self.salt);
        claims.insert("app".into(), self.app);
        claims.insert("role".into(), self.role);
        claims.insert("allow".into(), self.allow);
        claims.insert("sys".into(), self.is_sys_admin);

        claims
    }

    pub async fn generate(self) -> Option<String> {
        // let jwt_manager = provide::<JWTManager>().await;

        // jwt_manager.sign_to_jwt(self.build())
        unimplemented!()
    }
}

#[derive(Debug, Default)]
pub struct Claim {
    pub user_id: String,
    pub salt: String,
    pub app: String,
    pub role: String,
    pub allow: String,
}

impl From<HashMap<String, String>> for Claim {
    fn from(value: HashMap<String, String>) -> Self {
        let mut instance = Self::default();
        for entry in value.into_iter() {
            match entry.0.as_str() {
                "user" => instance.user_id = entry.1,
                "salt" => instance.salt = entry.1,
                "app" => instance.app = entry.1,
                "role" => instance.role = entry.1,
                "allow" => instance.allow = entry.1,
                _ => (),
            }
        }

        instance
    }
}
