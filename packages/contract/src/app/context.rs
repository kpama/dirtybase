use std::{ops::Deref, sync::Arc};

use anyhow::anyhow;
use axum::{extract::FromRequestParts, http::request::Parts};
use serde::de::DeserializeOwned;

mod app_context;
mod context_manager;
mod context_metadata;
mod role_context;
mod user_context;

use crate::{
    config::{DirtyConfig, TryFromDirtyConfig},
    multitenant::*,
};
pub use app_context::*;
pub use context_manager::*;
pub use context_metadata::*;
pub use role_context::*;
pub use user_context::*;

use crate::db::types::ArcUuid7;

#[derive(Clone)]
pub struct Context {
    id: ArcUuid7,
    is_global: bool,
    sc: Arc<busybody::ServiceContainer>,
}

impl Default for Context {
    fn default() -> Self {
        let instance = Self {
            id: ArcUuid7::default(),
            is_global: false,
            sc: Arc::new(busybody::helpers::make_proxy()),
        };

        instance
    }
}

impl Context {
    pub async fn make_global() -> Self {
        // app
        busybody::helpers::set_type(AppContext::make_global()).await;
        // role
        busybody::helpers::set_type(RoleContext::make_global()).await;
        // tenant
        busybody::helpers::set_type(TenantContext::make_global()).await;
        // user
        busybody::helpers::set_type(UserContext::make_global()).await;

        let instance = Self {
            id: ArcUuid7::default(),
            is_global: true,
            sc: busybody::helpers::service_container(),
        };

        busybody::helpers::set_type(instance.clone()).await;
        instance
    }

    pub async fn set_user(&self, user: UserContext) -> &Self {
        self.set(Arc::new(user)).await
    }

    pub async fn set_role(&self, role: RoleContext) -> &Self {
        self.set(Arc::new(role)).await
    }

    pub async fn set_tenant(&self, tenant: TenantContext) -> &Self {
        self.set(Arc::new(tenant)).await
    }

    pub async fn set_app(&self, app: AppContext) -> &Self {
        self.set(Arc::new(app)).await
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }

    pub async fn get_config<C>(&self, key: &str) -> Result<C, anyhow::Error>
    where
        C: DeserializeOwned + TryFromDirtyConfig<Returns = C>,
    {
        if let Some(app) = self.get::<AppContext>().await {
            if let Some(str_value) = app.config_string(key).await {
                return serde_json::from_str(&str_value).map_err(|e| anyhow!("{}", e));
            }
        }

        if let Some(dirty_config) = self.get::<DirtyConfig>().await {
            return C::from_config(&dirty_config).await;
        }

        Err(anyhow!("could not resolve configuration"))
    }

    pub fn container(&self) -> Arc<busybody::ServiceContainer> {
        self.sc.clone()
    }

    pub fn container_ref(&self) -> &Arc<busybody::ServiceContainer> {
        &self.sc
    }

    pub async fn user(&self) -> Option<UserContext> {
        self.get().await
    }

    pub async fn tenant(&self) -> Option<TenantContext> {
        self.get().await
    }

    pub async fn app(&self) -> Option<AppContext> {
        self.get().await
    }

    pub async fn role(&self) -> Option<RoleContext> {
        self.get().await
    }

    pub async fn set<T: Clone + Send + Sync + 'static>(&self, value: T) -> &Self {
        self.sc.set_type(value).await;
        self
    }

    pub async fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        let result = self.sc.get_type().await;

        if result.is_some() {
            return result;
        }

        ContextResourceManager::try_get(&self).await
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn id_ref(&self) -> &ArcUuid7 {
        &self.id
    }

    pub async fn metadata(&self) -> Arc<ContextMetadata> {
        let result = self.get().await;
        if result.is_none() {
            return self
                .set(Arc::new(ContextMetadata::default()))
                .await
                .get::<Arc<ContextMetadata>>()
                .await
                .unwrap();
        }
        result.unwrap()
    }
}

#[derive(Clone)]
#[must_use]
pub struct RequestContext(pub Context);

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = String;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            parts
                .extensions
                .get::<Context>()
                .ok_or_else(|| "Context not yet setup".to_string())
                .cloned()?,
        ))
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

        if let Some(ext) = context.get::<T>().await {
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
