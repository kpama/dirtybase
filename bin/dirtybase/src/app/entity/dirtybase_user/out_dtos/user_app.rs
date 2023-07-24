#[derive(Debug, Clone, serde::Serialize)]
pub struct UserApp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_system_app: String,
    pub company: String, // for now
}
