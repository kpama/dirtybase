use busybody::Injectable;

// TODO: remove serde::Serialization. This data should never leave the backend
/// Incoming DTO when the user is attempting to login
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserLoginPayload {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

impl Default for UserLoginPayload {
    fn default() -> Self {
        Self {
            username: None,
            email: None,
            password: "".to_string(),
        }
    }
}

#[busybody::async_trait]
impl Injectable for UserLoginPayload {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        container.proxy_value().unwrap_or_default()
    }
}
