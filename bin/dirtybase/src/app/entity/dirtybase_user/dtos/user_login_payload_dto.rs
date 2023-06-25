// TODO: remove serde::Serialization. This data should never leave the backend
/// Incoming DTO when the user is attempting to login
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserLoginPayload {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

impl Default for UserLoginPayload {
    fn default() -> Self {
        Self {
            username: None,
            email: None,
            password: None,
        }
    }
}
