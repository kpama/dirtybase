use super::{hash_password, USER_TABLE};
use crate::base::{helper::generate_ulid, manager::Manager};
use std::env;

#[derive(Debug)]
pub struct UserEntity {
    pub internal_id: Option<u64>,
    pub id: String,
    pub username: String,
    pub email: String,
    password: String,
}

impl Default for UserEntity {
    fn default() -> Self {
        Self {
            internal_id: None,
            id: generate_ulid(),
            username: "".into(),
            email: "".into(),
            password: "".into(), // MUST be the hashed of the raw password
        }
    }
}

impl UserEntity {
    pub fn from_env() -> Self {
        let mut user = Self::default();

        if let Ok(username) = env::var("DTY_SYS_ADMIN_USERNAME") {
            if !username.is_empty() {
                user.username = username;
            }
        }
        if let Ok(email) = env::var("DTY_SYS_ADMIN_EMAIL") {
            if !email.is_empty() {
                user.email = email;
            }
        }

        if let Ok(password) = env::var("DTY_SYS_ADMIN_PASSWORD") {
            if !password.is_empty() {
                user.password = hash_password(&password);
            }
        }

        user
    }

    pub fn hashed_password(&self) -> String {
        self.password.clone()
    }

    pub fn set_password(&mut self, raw: &str) {
        self.password = hash_password(raw);
    }

    pub async fn exist(&self, manager: &mut Manager) -> bool {
        let result = manager
            .select_from_table(USER_TABLE, |q| {
                q.select("*");
            })
            .fetch_all_as_field_value()
            .await;

        println!("{:#?}", &result);
        println!("{:#?}", serde_json::to_string(&result.unwrap()).unwrap());

        return !manager
            .select_from_table(USER_TABLE, |q| {
                q.select("id");
                if !self.id.is_empty() {
                    q.eq("id", &self.id);
                } else {
                    q.eq("username", &self.username).eq("email", &self.email);
                }
            })
            .fetch_all_as_json()
            .await
            .unwrap()
            .is_empty();
    }
}
