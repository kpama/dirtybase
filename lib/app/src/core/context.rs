use std::sync::Arc;

use dirtybase_user::entity::user::UserStatus;

pub struct Context {
    user: Option<UserContext>,
    service_container: Arc<busybody::ServiceContainer>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            user: None,
            service_container: Arc::new(busybody::helpers::make_proxy()),
        }
    }
}

impl Context {
    pub fn service_container(&self) -> Arc<busybody::ServiceContainer> {
        self.service_container.clone()
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
    id: String,
    username: String,
    status: UserStatus,
    role: RoleContext,
    app: AppContext,
}

#[derive(Debug, Clone)]
pub struct RoleContext {
    id: String,
    name: String,
}

#[derive(Debug, Clone)]
pub struct CompanyContext {
    id: String,
    name: String,
}

#[derive(Debug, Clone)]
pub struct AppContext {
    id: String,
    name: String,
    company: CompanyContext,
}
