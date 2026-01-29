use anyhow::Context as AnyhowCtx;
use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use dirtybase_contract::{
    http_contract::HttpContext,
    multitenant_contract::{
        TenantResolvedMiddleware, TenantStorage, TenantStorageProvider,
        model::{FetchTenantOption, FetchTenantPayload, TenantId},
    },
    prelude::Context,
};

use crate::MultitenantConfig;

#[derive(Clone)]
pub struct MultiTenantManager {
    config: Arc<MultitenantConfig>,
    storage: TenantStorageProvider,
}

pub enum TenantInjectionError {
    TenantNotFound,
    SystemError(String),
}

impl Display for TenantInjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TenantNotFound => writeln!(f, "tenant now found"),
            Self::SystemError(err) => writeln!(f, "{}", err),
        }
    }
}

impl Debug for TenantInjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)
    }
}

impl std::error::Error for TenantInjectionError {}

impl MultiTenantManager {
    pub fn new(config: MultitenantConfig, storage: TenantStorageProvider) -> Self {
        Self {
            config: Arc::new(config),
            storage,
        }
    }

    pub async fn inject_tenant(
        &self,
        context: &Context,
        is_http: bool,
    ) -> Result<(), TenantInjectionError> {
        if !self.config.is_enabled() {
            tracing::debug!("mutitenant is disabled from the config");
            return Ok(());
        }

        let fetch_payload = if is_http {
            self.build_payload_from_http(context)
                .await
                .map_err(|e| TenantInjectionError::SystemError(e.to_string()))?
        } else {
            self.build_payload_from_cli(context)
                .await
                .map_err(|e| TenantInjectionError::SystemError(e.to_string()))?
        };

        let result = self
            .storage
            .find(fetch_payload, Some(FetchTenantOption::only_active()))
            .await;

        match result {
            Ok(Some(tenant)) => {
                tracing::debug!("got tenant: {:#?}", &tenant);
                // TODO: Announce that a new tenant has been set to the context
                if let Ok(http_ctx) = context.get::<HttpContext>().await {
                    http_ctx
                        .set_cookie_kv(self.config.cookie_key(), tenant.id().unwrap().to_string())
                        .await;
                }
                match TenantResolvedMiddleware::get().await.send(tenant).await {
                    Ok(tenant) => {
                        context.set(tenant).await;
                    }
                    Err(e) => return Err(TenantInjectionError::SystemError(e.to_string())),
                }
            }
            Ok(None) => {
                if self.config.tenant_require() {
                    return Err(TenantInjectionError::TenantNotFound);
                }
            }
            Err(e) => {
                return Err(TenantInjectionError::SystemError(e.to_string()));
            }
        }

        Ok(())
    }

    async fn build_payload_from_cli(
        &self,
        _context: &Context,
    ) -> Result<FetchTenantPayload, TenantInjectionError> {
        // FIXME: Get the tenant ID from the cli params
        unimplemented!()
    }

    async fn build_payload_from_http(
        &self,
        context: &Context,
    ) -> Result<FetchTenantPayload, anyhow::Error> {
        let http_ctx = context
            .get::<HttpContext>()
            .await
            .expect("could not fetch http context from context");

        Ok(
            if let Ok(payload) = context.get::<FetchTenantPayload>().await {
                payload
            } else {
                // 1. Check for tenant's ID in header
                if let Some(id) = http_ctx.header(self.config.header_key()) {
                    let id = TenantId::try_from(id.to_str().unwrap_or_else(|_| "--wrong--"))
                        .context("tenant's ID in header is not valid")?;
                    FetchTenantPayload::by_id(id)
                // 2. Check for tenant's token in header
                } else if let Some(token) = http_ctx.header(self.config.token_header_key()) {
                    let token = token.to_str().unwrap_or_default();
                    if token.is_empty() {
                        return Err(anyhow::anyhow!("tenant's token in header is empty"));
                    }
                    FetchTenantPayload::by_token(token)
                // 3. Check for tenant's ID in query string
                } else if let Some(id) = http_ctx.get_a_query_by::<String>(self.config.query_key())
                {
                    if id.is_empty() {
                        return Err(anyhow::anyhow!("tenant's ID in header is emapty"));
                    }

                    FetchTenantPayload::by_id(
                        TenantId::try_from(id).context("tenant ID in query string is not valid")?,
                    )
                // 4. Check for tenant's token in query string
                } else if let Some(token) =
                    http_ctx.get_a_query_by::<String>(self.config.token_query_key())
                {
                    if token.is_empty() {
                        return Err(anyhow::anyhow!("tenant's token on query string is empty"));
                    }

                    FetchTenantPayload::by_token(token)
                    // 5. Check cookie
                } else if let Some(cookie) =
                    http_ctx.get_cookie_value(self.config.cookie_key()).await
                {
                    let id =
                        TenantId::try_from(cookie).context("tenant's ID in cookie is not valid")?;
                    FetchTenantPayload::ById { id: id }
                // 6. Check domain
                } else {
                    let domain = http_ctx.domain().unwrap_or_default();
                    if domain.is_empty() {
                        return Err(anyhow::anyhow!("tenant's domain is empty"));
                    }
                    FetchTenantPayload::ByDomain {
                        domain: domain.into(),
                    }
                }
            },
        )
    }

    pub fn config(&self) -> &MultitenantConfig {
        &self.config
    }
}
