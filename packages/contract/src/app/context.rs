use std::sync::Arc;

use crate::{db::types::ArcUlidField, user::status::UserStatus};

pub struct Context {
    id: ArcUlidField,
    user: Option<UserContext>,
    service_container: Arc<busybody::ServiceContainer>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            id: ArcUlidField::default(),
            user: None,
            service_container: Arc::new(busybody::helpers::make_proxy()),
        }
    }
}

impl Context {
    pub fn service_container(&self) -> Arc<busybody::ServiceContainer> {
        self.service_container.clone()
    }

    pub fn service_container_ref(&self) -> &Arc<busybody::ServiceContainer> {
        &self.service_container
    }

    pub fn has_user(&self) -> bool {
        self.user.is_some()
    }

    pub fn user(&self) -> Option<UserContext> {
        self.user.clone()
    }

    pub fn user_ref(&self) -> Option<&UserContext> {
        self.user.as_ref()
    }
    pub fn id(&self) -> ArcUlidField {
        self.id.clone()
    }
    pub fn id_ref(&self) -> &ArcUlidField {
        &self.id
    }

    pub fn set<T: Send + Sync + 'static>(&self, value: T) -> &Self {
        self.service_container.set(value);
        self
    }

    pub fn get<T: 'static>(&self) -> Option<busybody::Service<T>> {
        self.service_container.get()
    }
}

impl Context {
    pub fn new(user: Option<UserContext>) -> Self {
        Self {
            user,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserContext {
    id: ArcUlidField,
    username: String,
    status: UserStatus,
    role: RoleContext,
    app: Option<AppContext>,
}

impl UserContext {
    pub fn id(&self) -> ArcUlidField {
        self.id.clone()
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn status(&self) -> UserStatus {
        self.status.clone()
    }

    pub fn role(&self) -> &RoleContext {
        &self.role
    }

    pub fn app(&self) -> &Option<AppContext> {
        &self.app
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
}
