use std::{str::FromStr, sync::Arc};

use dirtybase_helper::uuid::Uuid25;

use crate::{db_contract::types::ArcUuid7, http_contract::HttpContext};

pub const TENANT_ID_HEADER: &str = "X-Tenant-ID";
pub const TENANT_ID_QUERY_STRING: &str = "tenant-id";

/// Tenant ID default location
///
/// When the current user has not been authenticated,
/// the application will check one of these locations
/// for the tenant ID.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum TenantIdLocation {
    /// Domain are tenant specific
    #[serde(alias = "domain")]
    Domain,
    /// Subdomain per tenant
    #[serde(alias = "subdomain")]
    Subdomain,
    /// Tenant ID in header under X-Tenant-ID
    #[serde(alias = "header")]
    Header,
    /// Tenant ID is in the query string as `tenant-id`
    #[serde(alias = "query")]
    Query,
}

impl Default for TenantIdLocation {
    fn default() -> Self {
        Self::Subdomain
    }
}

#[async_trait::async_trait]
pub trait RequestTenantResolverTrait: Send + Sync {
    /// Try finding the raw (string) tenant ID from the request
    async fn pluck_id_str_from_request(
        &self,
        http_ctx: &HttpContext,
        id_location: TenantIdLocation,
    ) -> Option<String> {
        match id_location {
            TenantIdLocation::Subdomain => http_ctx.subdomain().map(|x| x.to_string()),
            TenantIdLocation::Domain => http_ctx.host(),
            TenantIdLocation::Header => {
                if let Some(id) = http_ctx.header(TENANT_ID_HEADER) {
                    if let Ok(s) = id.to_str() {
                        return Some(s.to_string());
                    }
                }
                None
            }
            TenantIdLocation::Query => http_ctx.get_a_query_by::<String>(TENANT_ID_QUERY_STRING),
        }
    }
    /// Try plucking the tenant's ID from the request
    ///
    /// Usually as this stage the user has not being authenticated
    async fn id_from_request(
        &self,
        http_ctx: &HttpContext,
        id_location: TenantIdLocation,
    ) -> Option<ArcUuid7> {
        self.pluck_id_str_from_request(http_ctx, id_location)
            .await
            .and_then(|id| ArcUuid7::try_from(id.as_str()).ok())
    }

    ///  Encode and return the tenant's ID in a format suitable for the web
    async fn url_encode_id(&self, id: ArcUuid7) -> String {
        id.to_uuid25_string()
    }

    /// Try decoding the ID received from the web
    async fn url_decode_id(&self, id: &str) -> Option<ArcUuid7> {
        match Uuid25::from_str(id) {
            Ok(u25) => Some(ArcUuid7::from(u25)),
            Err(e) => {
                if let Ok(id) = ArcUuid7::try_from(id) {
                    return Some(id);
                }

                tracing::error!("could not decode tenant id '{}'. {}", id, e.to_string());
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestTenantResolver;

#[async_trait::async_trait]
impl RequestTenantResolverTrait for RequestTenantResolver {}

#[derive(Clone)]
pub struct RequestTenantResolverProvider(Arc<Box<dyn RequestTenantResolverTrait>>);

#[async_trait::async_trait]
impl RequestTenantResolverTrait for RequestTenantResolverProvider {
    async fn pluck_id_str_from_request(
        &self,
        http_ctx: &HttpContext,
        id_location: TenantIdLocation,
    ) -> Option<String> {
        self.0
            .pluck_id_str_from_request(http_ctx, id_location)
            .await
    }
    async fn id_from_request(
        &self,
        http_ctx: &HttpContext,
        id_location: TenantIdLocation,
    ) -> Option<ArcUuid7> {
        self.0.id_from_request(http_ctx, id_location).await
    }

    async fn url_encode_id(&self, id: ArcUuid7) -> String {
        self.0.url_encode_id(id).await
    }

    async fn url_decode_id(&self, id: &str) -> Option<ArcUuid7> {
        self.0.url_decode_id(id).await
    }
}

impl Default for RequestTenantResolverProvider {
    fn default() -> Self {
        Self(Arc::new(Box::new(RequestTenantResolver)))
    }
}

impl RequestTenantResolverProvider {
    pub fn new(inner: impl RequestTenantResolverTrait + 'static) -> Self {
        Self(Arc::new(Box::new(inner)))
    }

    pub fn from<T>(inner: T) -> Self
    where
        T: RequestTenantResolverTrait + 'static,
    {
        Self::new(inner)
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
    use crate::multitenant_contract::{RequestTenantResolver, TENANT_ID_HEADER};
    use crate::multitenant_contract::{RequestTenantResolverTrait, TENANT_ID_QUERY_STRING};

    #[tokio::test]
    async fn test_wrong_uuid7_from_subdomain() {
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority("t1.awesomeapp.com")
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
                    .authority(format!("{}.awesomeapp.com", id)) // "0194ddfb-ed23-77af-ba48-cee803fbb0b5.awesomeapp.com"
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
                    .authority(format!("{}.awesomeapp.com", id)) // "0194ddfb-ed23-77af-ba48-cee803fbb0b5.awesomeapp.com"
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

        let uuid25_string = resolver.url_encode_id(uuid).await;
        println!("{}", &uuid25_string);
        assert_eq!(uuid25_string, Uuid25::from_str(id).unwrap().to_string());
    }

    #[tokio::test]
    async fn test_decoding_uuid7_str() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let uuid = ArcUuid7::try_from(id).expect("could not decode uuid7 string");
        let resolver = RequestTenantResolver;

        assert_eq!(resolver.url_decode_id(id).await.unwrap(), uuid);
    }

    #[tokio::test]
    async fn test_decoding_uuid25_str() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let uuid = ArcUuid7::try_from(id).expect("could not decode uuid7 string");
        let uuid25_str = Uuid25::from_str(id).unwrap().to_string();

        let resolver = RequestTenantResolver;
        assert_eq!(
            resolver.url_decode_id(uuid25_str.as_str()).await.unwrap(),
            uuid
        );
    }

    #[tokio::test]
    async fn test_wrong_uuid7_from_header() {
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority("awesomeapp.com")
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
                    .authority("awesomeapp.com") // "0194ddfb-ed23-77af-ba48-cee803fbb0b5.awesomeapp.com"
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
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority("awesomeapp.com")
                    .path_and_query(format!("/?{}=123", TENANT_ID_QUERY_STRING))
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
                    crate::multitenant_contract::TenantIdLocation::Query
                )
                .await,
            None
        );
    }

    #[tokio::test]
    async fn test_correct_uuid7_from_query() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let req = request::Builder::new()
            .uri(
                Uri::builder()
                    .scheme("http")
                    .authority("awesomeapp.com")
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
