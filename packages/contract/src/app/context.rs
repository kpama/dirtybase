use std::{fmt::Display, ops::Deref, sync::Arc};

use axum::{extract::FromRequestParts, http::request::Parts};
use busybody::Service;

mod context_metadata;
mod user_context;

pub use context_metadata::*;
pub use user_context::*;

#[derive(Clone)]
pub struct ContextId(Arc<String>);

impl Default for ContextId {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextId {
    pub fn new() -> Self {
        Self(Arc::new(dirtybase_helper::uuid::uuid25_v4_string()))
    }
}

impl Display for ContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Clone)]
pub struct Context {
    id: ContextId,
    sc: Arc<busybody::ServiceContainer>,
}

impl Default for Context {
    fn default() -> Self {
        let instance = Self {
            id: ContextId::default(),
            sc: Arc::new(busybody::helpers::make_proxy()),
        };

        instance.set(ContextMetadata::default());
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

    pub fn id(&self) -> ContextId {
        self.id.clone()
    }

    pub fn id_ref(&self) -> &String {
        &self.id.0
    }

    pub fn metadata(&self) -> Service<ContextMetadata> {
        self.get().unwrap()
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

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let context = parts
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
