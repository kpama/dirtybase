use crate::{db::types::ArcUlidField, user::status::UserStatus};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UserContext {
    id: ArcUlidField,
    username: String,
    status: UserStatus,
    role: RoleContext,
    app: AppContext,
}

impl UserContext {
    pub fn id(&self) -> ArcUlidField {
        self.id.clone()
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn status(&self) -> UserStatus {
        self.status
    }

    pub fn role(&self) -> &RoleContext {
        &self.role
    }

    pub fn app(&self) -> &AppContext {
        &self.app
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RoleContext {
    id: ArcUlidField,
    name: String,
}

impl RoleContext {
    pub fn id(&self) -> ArcUlidField {
        self.id.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CompanyContext {
    id: ArcUlidField,
    name: String,
}

impl CompanyContext {
    pub fn id(&self) -> ArcUlidField {
        self.id.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppContext {
    id: ArcUlidField,
    name: String,
    company: CompanyContext,
}

impl AppContext {
    pub fn id(&self) -> ArcUlidField {
        self.id.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn company(&self) -> &CompanyContext {
        &self.company
    }
}
