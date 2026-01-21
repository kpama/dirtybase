use anyhow::Context as AnyhowCtx;

use crate::{
    http_contract::HttpContext,
    multitenant_contract::{
        model::{FetchTenantOption, FetchTenantPayload, Tenant, TenantId},
        TenantStorage, TenantStorageProvider, TENANT_ID_HEADER, TENANT_ID_QUERY_STRING,
        TENANT_TOKEN_HEADER, TENANT_TOKEN_QUERY_STRING,
    },
    prelude::Context,
};

#[derive(Clone)]
pub struct TenantManager {
    storage: TenantStorageProvider,
}

impl TenantManager {
    pub fn new(storage: TenantStorageProvider) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> &TenantStorageProvider {
        &self.storage
    }

    pub async fn find_tenant_from_http(
        &self,
        context: &Context,
    ) -> Result<Option<Tenant>, anyhow::Error> {
        let http_ctx = context
            .get::<HttpContext>()
            .await
            .expect("could not fetch http context from context");
        let fetch_payload = if let Ok(payload) = context.get::<FetchTenantPayload>().await {
            payload
        } else {
            // 1. Check for tenant's ID in header
            if let Some(id) = http_ctx.header(TENANT_ID_HEADER) {
                let id = TenantId::try_from(id.to_str().unwrap_or_else(|_| "--wrong--"))
                    .context("tenant's ID in header is not valid")?;
                FetchTenantPayload::by_id(id)
            // 2. Check for tenant's token in header
            } else if let Some(token) = http_ctx.header(TENANT_TOKEN_HEADER) {
                let token = token.to_str().unwrap_or_default();
                if token.is_empty() {
                    return Err(anyhow::anyhow!("tenant's token in header is empty"));
                }
                FetchTenantPayload::by_token(token)
            // 3. Check for tenant's ID in query string
            } else if let Some(id) = http_ctx.get_a_query_by::<String>(TENANT_ID_QUERY_STRING) {
                if id.is_empty() {
                    return Err(anyhow::anyhow!("tenant's ID in header is emapty"));
                }

                FetchTenantPayload::by_id(
                    TenantId::try_from(id).context("tenant ID in query string is not valid")?,
                )
            // 4. Check for tenant's token in query string
            } else if let Some(token) = http_ctx.get_a_query_by::<String>(TENANT_TOKEN_QUERY_STRING)
            {
                if token.is_empty() {
                    return Err(anyhow::anyhow!("tenant's token on query string is empty"));
                }

                FetchTenantPayload::by_token(token)
            // 5. Check domain
            } else {
                let domain = http_ctx.domain().unwrap_or_default();
                if domain.is_empty() {
                    return Err(anyhow::anyhow!("tenant's domain is empty"));
                }
                FetchTenantPayload::ByDomain {
                    domain: domain.into(),
                }
            }
        };

        self.storage
            .find(fetch_payload, Some(FetchTenantOption::only_active()))
            .await
    }
}
