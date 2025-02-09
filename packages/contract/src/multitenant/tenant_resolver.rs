use std::{str::FromStr, sync::Arc};

use axum::{body::Body, http::Request};
use dirtybase_helper::uuid::Uuid25;

use crate::db::types::ArcUuid7;

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
pub trait TenantResolverTrait: Send + Sync {
    /// Try finding the raw (string) tenant ID from the request
    fn pluck_id_str_from_request(
        &self,
        req: &Request<Body>,
        id_location: TenantIdLocation,
    ) -> Option<String> {
        return match id_location {
            TenantIdLocation::Subdomain => {
                if let Some(domain) = crate::http::axum::host_from_request(req) {
                    if let Some(id) = domain.split(".").next() {
                        return Some(id.to_string());
                    }
                }
                None
            }
            TenantIdLocation::Domain => None,
            TenantIdLocation::Header => {
                if let Some(id) = req.headers().get(TENANT_ID_HEADER) {
                    if let Ok(s) = id.to_str() {
                        return Some(s.to_string());
                    }
                }
                None
            }
            TenantIdLocation::Query => {
                crate::http::axum::query_to_kv(req.uri().query().unwrap_or_default())
                    .get(TENANT_ID_QUERY_STRING)
                    .cloned()
            }
        };
    }
    /// Try plucking the tenant's ID from the request
    ///
    /// Usually as this stage the user has not being authenticated
    fn id_from_request(
        &self,
        req: &Request<Body>,
        id_location: TenantIdLocation,
    ) -> Option<ArcUuid7> {
        self.pluck_id_str_from_request(req, id_location)
            .and_then(|id| ArcUuid7::try_from(id.as_str()).ok())
    }

    ///  Encode and return the tenant's ID in a format suitable for the web
    async fn url_encode_id(&self, id: ArcUuid7) -> String {
        id.to_uuid25_string()
    }

    /// Try decoding the ID received from the web
    async fn url_decode_id(&self, id: &str) -> Option<ArcUuid7> {
        match Uuid25::from_str(id) {
            Ok(u25) => ArcUuid7::try_from(u25).ok(),
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
pub struct TenantResolver;

#[async_trait::async_trait]
impl TenantResolverTrait for TenantResolver {}

#[derive(Clone)]
pub struct TenantResolverProvider(Arc<Box<dyn TenantResolverTrait>>);

#[async_trait::async_trait]
impl TenantResolverTrait for TenantResolverProvider {
    fn pluck_id_str_from_request(
        &self,
        req: &Request<Body>,
        id_location: TenantIdLocation,
    ) -> Option<String> {
        self.0.pluck_id_str_from_request(req, id_location)
    }
    fn id_from_request(
        &self,
        req: &Request<Body>,
        id_location: TenantIdLocation,
    ) -> Option<ArcUuid7> {
        self.0.id_from_request(req, id_location)
    }

    async fn url_encode_id(&self, id: ArcUuid7) -> String {
        self.0.url_encode_id(id).await
    }

    async fn url_decode_id(&self, id: &str) -> Option<ArcUuid7> {
        self.0.url_decode_id(id).await
    }
}

impl Default for TenantResolverProvider {
    fn default() -> Self {
        Self(Arc::new(Box::new(TenantResolver)))
    }
}

impl TenantResolverProvider {
    pub fn new(inner: impl TenantResolverTrait + 'static) -> Self {
        Self(Arc::new(Box::new(inner)))
    }

    pub fn from<T>(inner: T) -> Self
    where
        T: TenantResolverTrait + 'static,
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

    use crate::db::types::ArcUuid7;
    use crate::multitenant::{TenantResolver, TENANT_ID_HEADER};
    use crate::multitenant::{TenantResolverTrait, TENANT_ID_QUERY_STRING};

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

        let resolver = TenantResolver;

        assert_eq!(
            resolver.id_from_request(&req, crate::multitenant::TenantIdLocation::Subdomain),
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

        let resolver = TenantResolver;

        assert_eq!(
            resolver.id_from_request(&req, crate::multitenant::TenantIdLocation::Subdomain),
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

        let resolver = TenantResolver;
        let uuid = resolver
            .id_from_request(&req, crate::multitenant::TenantIdLocation::Subdomain)
            .unwrap();

        let uuid25_string = resolver.url_encode_id(uuid).await;
        println!("{}", &uuid25_string);
        assert_eq!(uuid25_string, Uuid25::from_str(id).unwrap().to_string());
    }

    #[tokio::test]
    async fn test_decoding_uuid7_str() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let uuid = ArcUuid7::try_from(id).expect("could not decode uuid7 string");
        let resolver = TenantResolver;

        assert_eq!(resolver.url_decode_id(id).await.unwrap(), uuid);
    }

    #[tokio::test]
    async fn test_decoding_uuid25_str() {
        let id = "0194ddfb-ed23-77af-ba48-cee803fbb0b5";
        let uuid = ArcUuid7::try_from(id).expect("could not decode uuid7 string");
        let uuid25_str = Uuid25::from_str(id).unwrap().to_string();

        let resolver = TenantResolver;
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

        let resolver = TenantResolver;

        assert_eq!(
            resolver.id_from_request(&req, crate::multitenant::TenantIdLocation::Header),
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

        let resolver = TenantResolver;

        assert_eq!(
            resolver.id_from_request(&req, crate::multitenant::TenantIdLocation::Header),
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

        let resolver = TenantResolver;

        assert_eq!(
            resolver.id_from_request(&req, crate::multitenant::TenantIdLocation::Query),
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

        let resolver = TenantResolver;

        assert_eq!(
            resolver.id_from_request(&req, crate::multitenant::TenantIdLocation::Query),
            ArcUuid7::try_from(id).ok()
        );
    }
}
