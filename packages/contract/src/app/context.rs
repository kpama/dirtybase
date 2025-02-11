use std::{ops::Deref, sync::Arc};

use axum::{extract::FromRequestParts, http::request::Parts};
use serde::de::DeserializeOwned;

mod app_context;
mod context_manager;
mod context_metadata;
mod role_context;
mod user_context;

use crate::multitenant::*;
pub use app_context::*;
pub use context_manager::*;
pub use context_metadata::*;
pub use role_context::*;
pub use user_context::*;

use crate::db::types::ArcUuid7;

pub const GLOBAL_CONTEXT_ID: &str = "0194d467-2006-75f0-9fe7-575ec6e00b79";

#[derive(Clone)]
pub struct Context {
    id: ArcUuid7,
    sc: Arc<busybody::ServiceContainer>,
}

impl Default for Context {
    fn default() -> Self {
        let instance = Self {
            id: ArcUuid7::default(),
            sc: Arc::new(busybody::helpers::make_proxy()),
        };

        instance.set(Arc::new(ContextMetadata::default()));
        instance
    }
}

impl Context {
    pub fn make_global() -> Self {
        // app
        busybody::helpers::set_type(Arc::new(AppContext::make_global()));
        // role
        busybody::helpers::set_type(Arc::new(RoleContext::make_global()));
        // tenant
        busybody::helpers::set_type(Arc::new(TenantContext::make_global()));
        // user
        busybody::helpers::set_type(Arc::new(UserContext::make_global()));

        Self {
            id: ArcUuid7::try_from(GLOBAL_CONTEXT_ID).unwrap(),
            sc: Arc::new(busybody::helpers::make_proxy()),
        }
    }

    pub fn set_user(&self, user: UserContext) -> &Self {
        self.set(Arc::new(user))
    }

    pub fn set_role(&self, role: RoleContext) -> &Self {
        self.set(Arc::new(role))
    }

    pub fn set_tenant(&self, tenant: TenantContext) -> &Self {
        self.set(Arc::new(tenant))
    }

    pub fn set_app(&self, app: AppContext) -> &Self {
        self.set(Arc::new(app))
    }

    pub fn is_global(&self) -> bool {
        self.id.to_string() == GLOBAL_CONTEXT_ID
    }

    pub fn configure<C>(&self, key: &str) -> Option<C>
    where
        C: DeserializeOwned + Sync + Send + 'static,
    {
        if let Some(tenant) = self.get::<TenantContext>() {
            if let Some(config) = tenant.config_to::<C>(key) {
                return Some(config);
            }
        }
        None
    }

    pub fn container(&self) -> Arc<busybody::ServiceContainer> {
        self.sc.clone()
    }

    pub fn container_ref(&self) -> &Arc<busybody::ServiceContainer> {
        &self.sc
    }

    pub fn user(&self) -> Option<Arc<UserContext>> {
        self.get()
    }

    pub fn tenant(&self) -> Option<Arc<TenantContext>> {
        self.get()
    }

    pub fn app(&self) -> Option<Arc<AppContext>> {
        self.get()
    }

    pub fn role(&self) -> Option<Arc<RoleContext>> {
        self.get()
    }

    pub fn set<T: Clone + Send + Sync + 'static>(&self, value: T) -> &Self {
        self.sc.set_type(value);
        self
    }

    pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.sc.get_type()
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn id_ref(&self) -> &ArcUuid7 {
        &self.id
    }

    pub fn metadata(&self) -> Arc<ContextMetadata> {
        self.get().unwrap()
    }
}

/// Current request context extension manager
///
/// When an extension is requested, the current request context is used
/// before falling back to the the global context
#[derive(Debug, Clone)]
#[must_use]
pub struct CtxExt<T>(pub T);

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
            .ok_or_else(|| "Context not yet setup".to_string())
            .cloned()?;

        if let Some(ext) = context.get::<T>() {
            Ok(Self(ext))
        } else {
            tracing::error!("{} not found in context", std::any::type_name::<T>());
            Err(format!(
                "{} not found in context",
                std::any::type_name::<T>()
            ))
        }
    }
}

impl<T> Deref for CtxExt<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
