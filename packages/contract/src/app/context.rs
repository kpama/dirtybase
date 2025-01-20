use std::{ops::Deref, sync::Arc};

use axum::{extract::FromRequestParts, http::request::Parts};
use busybody::Service;

use crate::{db::types::ArcUlidField, user::status::UserStatus};

#[derive(Clone)]
pub struct Context {
    sc: Arc<busybody::ServiceContainer>,
}

impl Default for Context {
    fn default() -> Self {
        let instance = Self {
            sc: Arc::new(busybody::helpers::make_proxy()),
        };
        instance
    }
}

impl Context {
    pub fn set_user(&self, user: UserContext) -> &Self {
        self.sc.set(user);
        self
    }

    pub fn service_container(&self) -> Arc<busybody::ServiceContainer> {
        self.sc.clone()
    }

    pub fn service_container_ref(&self) -> &Arc<busybody::ServiceContainer> {
        &self.sc
    }

    pub fn has_user(&self) -> bool {
        self.sc.get::<UserContext>().is_some()
    }

    pub fn user(&self) -> Option<Service<UserContext>> {
        self.sc.get::<UserContext>()
    }

    pub fn set<T: Send + Sync + 'static>(&self, value: T) -> &Self {
        self.sc.set(value);
        self
    }

    pub fn get<T: 'static>(&self) -> Option<Service<T>> {
        self.sc.get()
    }
}

#[derive(Debug, Clone, Default)]
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
        self.status.clone()
    }

    pub fn role(&self) -> &RoleContext {
        &self.role
    }

    pub fn app(&self) -> &AppContext {
        &self.app
    }
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
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

/// Current request context extension manager
///
/// When an extension is requested, the current request context is used
/// before falling back to the the global context
#[derive(Debug, Clone)]
#[must_use]
pub struct CtxExt<T>(pub Service<T>);

impl<T, S> FromRequestParts<S> for CtxExt<T>
where
    T: Clone + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = String;

    async fn from_request_parts(req: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let context = req
            .extensions
            .get::<Context>()
            .ok_or_else(|| "Error".to_string())
            .cloned()?;

        if let Some(ext) = context.get::<T>() {
            Ok(Self(ext))
        } else {
            Err("Erooor...".to_string())
        }
    }
}

impl<T> Deref for CtxExt<T> {
    type Target = Service<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
