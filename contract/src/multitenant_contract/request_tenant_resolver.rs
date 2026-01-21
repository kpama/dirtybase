use std::{future::Future, str::FromStr, sync::Arc};

use dirtybase_helper::uuid::Uuid25;
use simple_middleware::Manager;

use crate::{
    db_contract::types::ArcUuid7,
    http_contract::HttpContext,
    multitenant_contract::{
        model::{FetchTenantPayload, Tenant, TenantId},
        TenantStorage, TenantStorageProvider, MULTITENANT_CONTRACT_LOG,
    },
};

pub const TENANT_ID_HEADER: &str = "X-Tenant-ID";
pub const TENANT_TOKEN_HEADER: &str = "X-Tenant-Token";
pub const TENANT_ID_QUERY_STRING: &str = "tenant-id";
pub const TENANT_TOKEN_QUERY_STRING: &str = "tenant-tk";

/// Tenant ID default location
///
/// When the current user has not been authenticated,
/// the application will check one of these locations
/// for the tenant ID.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub enum TenantIdLocation {
    /// Domain are tenant specific
    #[serde(alias = "domain")]
    Domain,
    /// Subdomain per tenant
    #[serde(alias = "subdomain")]
    #[default]
    Subdomain,
    /// Tenant ID in header under X-Tenant-ID
    #[serde(alias = "header")]
    Header,
    /// Tenant ID is in the query string as `tenant-id`
    #[serde(alias = "query")]
    Query,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TenantIdentifier {
    Id(TenantId),
    Name(String),
    Token(String),
}

#[derive(Debug, Clone)]
pub struct RequestTenantResolver;

impl RequestTenantResolver {
    /// Make new instance
    pub fn new() -> Self {
        Self
    }
    /// Try finding the raw (string) tenant ID from the request
    pub async fn pluck_id_str_from_request(
        &self,
        http_ctx: &HttpContext,
        id_location: TenantIdLocation,
    ) -> Option<String> {
        match id_location {
            TenantIdLocation::Subdomain => {
                let sb = http_ctx.subdomain().map(|x| x.to_string());

                tracing::debug!(
                    target: MULTITENANT_CONTRACT_LOG,
                    "plucking tenant's ID from sub-domain: {}",
                    if let Some(sub) = &sb { sub } else { "unknown" }
                );
                sb
            }
            TenantIdLocation::Domain => {
                let host = http_ctx.host();
                tracing::debug!(
                    target: MULTITENANT_CONTRACT_LOG,
                    "plucking tenant's ID from host: {}",
                    if let Some(d) = &host { d } else { "unknown" }
                );
                host
            }
            TenantIdLocation::Header => {
                if let Some(id) = http_ctx.header(TENANT_ID_HEADER) {
                    let header = id.to_str();
                    tracing::debug!(
                        target: MULTITENANT_CONTRACT_LOG,
                        "plucking tenant's ID from header: {}",
                        if let Ok(h) = &header { h } else { "unknown" }
                    );
                    if let Ok(s) = header {
                        return Some(s.to_string());
                    }
                }
                tracing::error!(target: MULTITENANT_CONTRACT_LOG ,"could not pluck tenant's ID from the header");
                None
            }
            TenantIdLocation::Query => {
                let query = http_ctx.get_a_query_by::<String>(TENANT_ID_QUERY_STRING);
                tracing::debug!(
                    target = MULTITENANT_CONTRACT_LOG,
                    "plucking tenant's ID from query string: {}",
                    if let Some(id) = &query { id } else { "unknown" }
                );
                query
            }
        }
    }

    /// Try plucking the tenant's ID from the request
    ///
    /// Usually as this stage the user has not being authenticated
    pub async fn id_from_request(
        &self,
        http_ctx: &HttpContext,
        id_location: TenantIdLocation,
    ) -> Option<ArcUuid7> {
        self.pluck_id_str_from_request(http_ctx, id_location)
            .await
            .and_then(|id| ArcUuid7::try_from(id.as_str()).ok())
    }

    ///  Encode and return the tenant's ID in a format suitable for the web
    pub async fn url_encode_id(&self, id: TenantId) -> String {
        id.to_uuid25_string()
    }

    /// Try decoding the ID received from the web
    pub async fn url_decode_id(&self, id: &str) -> Option<TenantId> {
        match Uuid25::from_str(id) {
            Ok(u25) => Some(TenantId::from(ArcUuid7::from(u25))),
            Err(e) => {
                if let Ok(id) = ArcUuid7::try_from(id) {
                    return Some(TenantId::from(id));
                }

                tracing::error!("could not decode tenant id '{}'. {}", id, e.to_string());
                None
            }
        }
    }

    /// Try fetching the Tenant's record
    pub async fn get_tenant(
        &self,
        http_ctx: HttpContext,
        storage: TenantStorageProvider,
        location: TenantIdLocation,
    ) -> Result<Option<Tenant>, anyhow::Error> {
        Self::get_middleware()
            .await
            .send((self.clone(), http_ctx, location, storage))
            .await
    }

    /// Register a resolver
    pub async fn register<F, Fut>(location: TenantIdLocation, callback: F)
    where
        F: Clone + Fn(Self, HttpContext, TenantStorageProvider) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Option<Tenant>, anyhow::Error>> + Send + 'static,
    {
        Self::get_middleware()
            .await
            .next(move |(resolver, http_ctx, loc, storage), next| {
                let cb = callback.clone();
                let location = location.clone();
                async move {
                    if loc == location {
                        (cb)(resolver, http_ctx, storage).await
                    } else {
                        next.call((resolver, http_ctx, loc, storage)).await
                    }
                }
            })
            .await;
    }

    async fn get_middleware() -> Arc<
        Manager<
            (Self, HttpContext, TenantIdLocation, TenantStorageProvider),
            Result<Option<Tenant>, anyhow::Error>,
        >,
    > {
        if let Some(m) = busybody::helpers::get_service().await {
            m
        } else {
            let manager = Manager::<
                (Self, HttpContext, TenantIdLocation, TenantStorageProvider),
                Result<Option<Tenant>, anyhow::Error>,
            >::last(
                |(resolver, http_ctx, id_location, storage), _| async move {
                if let Some(id) = resolver
                    .id_from_request(&http_ctx, id_location.clone())
                    .await
                {
                    return storage
                        .find(
                            FetchTenantPayload::ById {
                                id: TenantId::from(id),
                            },
                            None,
                        )
                        .await;
                }

                Err(anyhow::anyhow!(format!(
                    "could not pluck tenant ID from: {:?}",
                    id_location
                )))
            }
            )
            .await;

            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap() // Should never fail as we just registered the instance
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use axum::body::Body;
    use axum::http::request::{self};
    use axum::http::{Extensions, Uri};
    use dirtybase_helper::uuid::Uuid25;

    use crate::db_contract::types::ArcUuid7;
    use crate::http_contract::HttpContext;
    use crate::multitenant_contract::TENANT_ID_QUERY_STRING;
    use crate::multitenant_contract::{RequestTenantResolver, TENANT_ID_HEADER};

    #[tokio::test]
    async fn test_wrong_uuid7_from_subdomain() {
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority("t1.awesome-app.com")
                    .path_and_query("/")
                    .build()
                    .unwrap(),
            )
            .extension(Extensions::default())
            .body(Body::empty())
            .unwrap();

        let resolver = RequestTenantResolver;
        let http_ctx = HttpContext::from_request(&req).await;

        assert_eq!(
            resolver
                .id_from_request(
                    &http_ctx,
                    crate::multitenant_contract::TenantIdLocation::Subdomain
                )
                .await,
            None
        );
    }

    #[tokio::test]
    async fn test_correct_uuid7_from_subdomain() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority(format!("{}.awesome-app.com", id)) // "0194ddfb-ed23-77af-ba48-cee803fbb0b5.awesome-app.com"
                    .path_and_query("/")
                    .build()
                    .unwrap(),
            )
            .extension(Extensions::default())
            .body(Body::empty())
            .unwrap();

        let resolver = RequestTenantResolver;
        let http_ctx = HttpContext::from_request(&req).await;

        assert_eq!(
            resolver
                .id_from_request(
                    &http_ctx,
                    crate::multitenant_contract::TenantIdLocation::Subdomain
                )
                .await,
            ArcUuid7::try_from(id).ok()
        );
    }

    #[tokio::test]
    async fn test_id_encoding_to_uuid25() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority(format!("{}.awesome-app.com", id)) // "0194ddfb-ed23-77af-ba48-cee803fbb0b5.awesome-app.com"
                    .path_and_query("/")
                    .build()
                    .unwrap(),
            )
            .extension(Extensions::default())
            .body(Body::empty())
            .unwrap();

        let resolver = RequestTenantResolver;
        let http_ctx = HttpContext::from_request(&req).await;
        let uuid = resolver
            .id_from_request(
                &http_ctx,
                crate::multitenant_contract::TenantIdLocation::Subdomain,
            )
            .await
            .unwrap();

        let uuid25_string = resolver.url_encode_id(uuid.into()).await;
        assert_eq!(uuid25_string, Uuid25::from_str(id).unwrap().to_string());
    }

    #[tokio::test]
    async fn test_decoding_uuid7_str() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let uuid = ArcUuid7::try_from(id).expect("could not decode uuid7 string");
        let resolver = RequestTenantResolver;

        assert_eq!(resolver.url_decode_id(id).await.unwrap(), uuid.into());
    }

    #[tokio::test]
    async fn test_decoding_uuid25_str() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let uuid = ArcUuid7::try_from(id).expect("could not decode uuid7 string");
        let uuid25_str = Uuid25::from_str(id).unwrap().to_string();

        let resolver = RequestTenantResolver;
        assert_eq!(
            resolver.url_decode_id(uuid25_str.as_str()).await.unwrap(),
            uuid.into()
        );
    }

    #[tokio::test]
    async fn test_wrong_uuid7_from_header() {
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority("awesome-app.com")
                    .path_and_query("/")
                    .build()
                    .unwrap(),
            )
            .header(TENANT_ID_HEADER, "123")
            .extension(Extensions::default())
            .body(Body::empty())
            .unwrap();

        let resolver = RequestTenantResolver;
        let http_ctx = HttpContext::from_request(&req).await;

        assert_eq!(
            resolver
                .id_from_request(
                    &http_ctx,
                    crate::multitenant_contract::TenantIdLocation::Header
                )
                .await,
            None
        );
    }

    #[tokio::test]
    async fn test_correct_uuid7_from_header() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority("awesome-app.com")
                    .path_and_query("/")
                    .build()
                    .unwrap(),
            )
            .header(TENANT_ID_HEADER, id)
            .extension(Extensions::default())
            .body(Body::empty())
            .unwrap();

        let resolver = RequestTenantResolver;
        let http_ctx = HttpContext::from_request(&req).await;

        assert_eq!(
            resolver
                .id_from_request(
                    &http_ctx,
                    crate::multitenant_contract::TenantIdLocation::Header
                )
                .await,
            ArcUuid7::try_from(id).ok()
        );
    }

    #[tokio::test]
    async fn test_wrong_uuid7_from_query() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority("awesome-app.com")
                    .path_and_query(format!("/?{}={}", TENANT_ID_QUERY_STRING, id))
                    .build()
                    .unwrap(),
            )
            .extension(Extensions::default())
            .body(Body::empty())
            .unwrap();

        let resolver = RequestTenantResolver;
        let http_ctx = HttpContext::from_request(&req).await;

        let result = resolver
            .id_from_request(
                &http_ctx,
                crate::multitenant_contract::TenantIdLocation::Query,
            )
            .await
            .unwrap();
        assert_eq!(result, ArcUuid7::try_from(id).unwrap());
    }

    #[tokio::test]
    async fn test_correct_uuid7_from_query() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority("awesome-app.com")
                    .path_and_query(format!("/?{}={}", TENANT_ID_QUERY_STRING, id))
                    .build()
                    .unwrap(),
            )
            .header(TENANT_ID_HEADER, id)
            .extension(Extensions::default())
            .body(Body::empty())
            .unwrap();

        let resolver = RequestTenantResolver;
        let http_ctx = HttpContext::from_request(&req).await;

        assert_eq!(
            resolver
                .id_from_request(
                    &http_ctx,
                    crate::multitenant_contract::TenantIdLocation::Query
                )
                .await,
            ArcUuid7::try_from(id).ok()
        );
    }
}
