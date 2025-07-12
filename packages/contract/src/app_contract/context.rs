use std::{ops::Deref, sync::Arc};

use anyhow::anyhow;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde::de::DeserializeOwned;

mod app_context;
mod context_manager;
mod context_metadata;

use crate::{
    auth_contract::AuthUser,
    config_contract::{DirtyConfig, TryFromDirtyConfig},
    http_contract::{Bind, ModelBindResolver},
    multitenant_contract::*,
};
pub use app_context::*;
pub use context_manager::*;
pub use context_metadata::*;

use crate::db_contract::types::ArcUuid7;

#[derive(Clone)]
pub struct Context {
    id: ArcUuid7,
    is_global: bool,
    sc: busybody::ServiceContainer,
}

impl Context {
    pub async fn new() -> Self {
        Self::new_with_id(ArcUuid7::default()).await
    }

    pub async fn new_with_id(id: ArcUuid7) -> Self {
        let sc = busybody::helpers::make_proxy();
        let instance = Self {
            id,
            is_global: false,
            sc: sc.clone(),
        };

        sc.set_type(instance.clone()).await;

        instance
    }

    pub async fn make_global() -> Self {
        // app
        busybody::helpers::set_type(AppContext::make_global()).await;
        // tenant
        busybody::helpers::set_type(TenantContext::make_global()).await;
        // user
        busybody::helpers::set_type(AuthUser::default()).await;

        let instance = Self {
            id: ArcUuid7::default(),
            is_global: true,
            sc: busybody::helpers::service_container(),
        };

        busybody::helpers::set_type(instance.clone()).await;
        instance
    }

    pub async fn set_user(&self, user: AuthUser) -> &Self {
        self.set(user).await
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
        if let Ok(app) = self.get::<AppContext>().await {
            if let Some(str_value) = app.config_string(key).await {
                return serde_json::from_str(&str_value).map_err(|e| anyhow!("{}", e));
            }
        }

        if let Ok(dirty_config) = self.get::<DirtyConfig>().await {
            return C::from_config(&dirty_config, self).await;
        }
        C::from_config(&DirtyConfig::new(), self).await
    }

    pub fn container(&self) -> busybody::ServiceContainer {
        self.sc.clone()
    }

    pub fn container_ref(&self) -> &busybody::ServiceContainer {
        &self.sc
    }

    pub async fn user(&self) -> Option<AuthUser> {
        self.get().await.ok()
    }

    pub async fn tenant(&self) -> Option<TenantContext> {
        self.get().await.ok()
    }

    pub async fn app(&self) -> Option<AppContext> {
        self.get().await.ok()
    }

    pub async fn set<T: Clone + Send + Sync + 'static>(&self, value: T) -> &Self {
        self.sc.set_type(value).await;
        self
    }

    pub async fn get<T: Clone + Send + Sync + 'static>(&self) -> Result<T, anyhow::Error> {
        let result = self.sc.get_type().await;

        if let Some(r) = result {
            return Ok(r);
        }

        ContextResourceManager::try_get(self).await
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn id_ref(&self) -> &ArcUuid7 {
        &self.id
    }

    pub async fn metadata(&self) -> Arc<ContextMetadata> {
        let result = self.get().await;
        if result.is_err() {
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
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(context) = parts.extensions.get::<Context>().cloned() else {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, String::new()));
        };

        if let Ok(ext) = context.get::<T>().await {
            return Ok(Self(ext));
        }

        if let Some(v) = context.container().get_type::<Bind<T>>().await {
            return Ok(Self(v.0));
        }

        match ModelBindResolver::new(context.clone(), None)
            .await
            .bind::<T>()
            .await
        {
            Ok(Some(bind)) => {
                return Ok(Self(
                    context.set(bind).await.get::<Bind<T>>().await.unwrap().0,
                ))
            }
            Ok(None) => return Err((StatusCode::NOT_FOUND, String::new())),
            _ => (),
        }

        tracing::error!("{} not found in context", std::any::type_name::<T>());
        Err((StatusCode::INTERNAL_SERVER_ERROR, String::new()))
    }
}

impl<T> Deref for CtxExt<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
