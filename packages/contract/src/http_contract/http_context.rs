use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use axum::{
    extract::{
        rejection::{PathRejection, QueryRejection},
        ConnectInfo, FromRequestParts, Path, Query, Request,
    },
    http::{header::USER_AGENT, request::Parts, HeaderMap, HeaderValue, Uri},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use dirtybase_helper::hash::sha256;
use serde::de::DeserializeOwned;
use tokio::sync::{Mutex, RwLock};

use super::axum::clone_request;

/// Provides common HTTP attributes for the current request
#[derive(Clone)]
pub struct HttpContext {
    uri: Uri,
    headers: Arc<HeaderMap<HeaderValue>>,
    parts: Arc<Mutex<Parts>>,
    ip: Option<IpAddr>,
    info: Option<ConnectInfo<SocketAddr>>,
    cookie_jar: Arc<RwLock<Option<CookieJar>>>,
    raw_path_value: Arc<HashMap<String, serde_json::Value>>,
    raw_query_value: Arc<HashMap<String, serde_json::Value>>,
    subdomain: Option<Arc<String>>,
}

impl HttpContext {
    pub async fn new<T, H: IntoIterator<Item = I>, I: ToString>(
        req: &Request<T>,
        trusted_headers: H,
        trusted_ips: &[TrustedIp],
    ) -> Self {
        let mut instance = Self::from_request(req).await;

        instance.ip = instance.ip_from_headers(trusted_headers, trusted_ips);

        instance
    }

    pub async fn from_request<T>(req: &Request<T>) -> Self {
        let mut p = clone_request(req).into_parts().0;

        let raw_path_value = if let Ok(Path(v)) =
            Path::<HashMap<String, serde_json::Value>>::from_request_parts(&mut p, &()).await
        {
            Arc::new(v)
        } else {
            Arc::new(HashMap::new())
        };

        let raw_query_value = if let Ok(Query(v)) =
            Query::<HashMap<String, serde_json::Value>>::from_request_parts(&mut p, &()).await
        {
            Arc::new(v)
        } else {
            Arc::new(HashMap::new())
        };

        let parts = Arc::new(Mutex::new(p));

        let subdomain = if let Some(host) = req.uri().host() {
            let pieces = host.split(".").map(String::from).collect::<Vec<String>>();
            if pieces.len() > 1 {
                pieces.first().cloned().map(Arc::new)
            } else {
                None
            }
        } else {
            None
        };

        Self {
            uri: req.uri().clone(),
            parts,
            raw_query_value,
            raw_path_value,
            subdomain,
            headers: Arc::new(req.headers().clone()),
            ip: None,
            info: req.extensions().get::<ConnectInfo<_>>().cloned(),
            cookie_jar: Arc::new(RwLock::new(Some(CookieJar::from_headers(req.headers())))),
        }
    }

    /// The request URI's path
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Tries to return the dynamic path(s) in the URI
    pub async fn get_path<T>(&self) -> Result<Path<T>, PathRejection>
    where
        T: DeserializeOwned + Send,
    {
        let mut lock = self.parts.lock().await;
        Path::<T>::from_request_parts(&mut lock, &()).await
    }

    /// Tries to return the query value in the URi
    pub async fn get_query<T>(&self) -> Result<Query<T>, QueryRejection>
    where
        T: DeserializeOwned + Send,
    {
        let mut lock = self.parts.lock().await;
        Query::<T>::from_request_parts(&mut lock, &()).await
    }

    /// Tries to return the dynamic path with the specified
    ///
    /// This is useful when you want just a path from the URI
    pub fn get_a_path_by<T>(&self, name: &str) -> Option<T>
    where
        T: DeserializeOwned + Send,
    {
        if let Some(value) = self.raw_path_value.get(name).cloned() {
            return serde_json::from_value::<T>(value).ok();
        }

        None
    }

    /// Tries to return a query value
    ///
    /// Useful when you want to pluck just a value
    pub fn get_a_query_by<T>(&self, name: &str) -> Option<T>
    where
        T: DeserializeOwned + Send,
    {
        if let Some(value) = self.raw_query_value.get(name).cloned() {
            return serde_json::from_value::<T>(value).ok();
        }

        None
    }

    /// Returns all the dynamic path names in the URI
    pub fn get_path_names(&self) -> Vec<String> {
        self.raw_path_value.keys().cloned().collect()
    }

    /// Returns all the query string names
    pub fn get_query_names(&self) -> Vec<String> {
        self.raw_query_value.keys().cloned().collect()
    }

    /// The current request client's user agent
    pub fn user_agent(&self) -> Option<HeaderValue> {
        self.headers.get(USER_AGENT).cloned()
    }

    pub fn header(&self, name: &str) -> Option<HeaderValue> {
        self.headers.get(name).cloned()
    }

    /// The generated fingerprint for the current request
    pub fn fingerprint(&self) -> String {
        let id = self.to_string();
        sha256::hash_str(&id)
    }

    /// Returns the full URL for the current request
    pub fn full_path(&self) -> String {
        let mut full_path = String::new();

        // http:// or https://
        if let Some(scheme) = self.uri.scheme_str() {
            full_path.push_str(&format!("{scheme}://",));
        }

        // foo.com or 127.0.0.1
        if let Some(host) = self.uri.host() {
            full_path.push_str(host);
        }

        // /home or /home?a=1&b=2
        if let Some(path_n_query) = self.uri.path_and_query() {
            full_path.push_str(path_n_query.as_str());
        }

        full_path
    }

    pub fn query(&self) -> Option<String> {
        self.uri.query().map(|q| q.to_string())
    }

    pub fn host(&self) -> Option<String> {
        if let Some(host) = self.uri.host() {
            return Some(host.to_string());
        }

        None
    }

    pub fn subdomain(&self) -> Option<Arc<String>> {
        self.subdomain.clone()
    }

    pub async fn get_cookie(&self, name: &str) -> Option<Cookie> {
        let r_lock = self.cookie_jar.read().await;
        r_lock.as_ref().unwrap().get(name).cloned()
    }

    pub async fn set_cookie(&self, cookie: Cookie<'static>) {
        let mut w_lock = self.cookie_jar.write().await;
        *w_lock = Some(w_lock.take().unwrap().add(cookie));
    }

    pub async fn cookie_jar(&self) -> CookieJar {
        let r_lock = self.cookie_jar.read().await;
        r_lock.as_ref().unwrap().clone()
    }

    pub async fn set_cookie_kv<V>(&self, name: &str, value: V)
    where
        V: ToString,
    {
        let mut cookie = Cookie::new(name.to_string(), value.to_string());
        cookie.set_path("/");
        self.set_cookie(cookie).await
    }

    pub fn query_as_map(&self) -> Option<HashMap<String, String>> {
        if let Some(query) = self.query() {
            let map = query
                .split("&")
                .map(|entry| entry.split("="))
                .map(|mut pieces| {
                    (
                        pieces.next().unwrap_or_default().to_string(),
                        pieces.next().unwrap_or_default().to_string(),
                    )
                })
                .filter(|kv| !kv.0.is_empty())
                .collect::<HashMap<String, String>>();
            if !map.is_empty() {
                return Some(map);
            }
        }

        None
    }

    pub fn ip(&self) -> Option<IpAddr> {
        self.ip
    }

    fn ip_from_headers<H: IntoIterator<Item = I>, I: ToString>(
        &self,
        trusted_headers: H,
        trusted: &[TrustedIp],
    ) -> Option<IpAddr> {
        let mut ip: Option<IpAddr> = None;
        let names = trusted_headers
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>();
        let accept_all = trusted
            .iter()
            .filter(|entry| **entry == TrustedIp::All)
            .count()
            > 0;

        if let Some(info) = &self.info {
            ip = Some(info.ip())
        }

        for a_name in &names {
            match a_name.to_lowercase().as_ref() {
                "x-forwarded-for" => {
                    if trusted.is_empty() {
                        continue;
                    }
                    if let Some(mut values) = self.x_forwarded_ips() {
                        ip = values.pop_front();
                        if ip.is_none() || accept_all {
                            continue;
                        }

                        if !values.is_empty() && ip.is_some() {
                            for forwarded_ip in &values {
                                for trusted_ip in trusted {
                                    if !trusted_ip.passes(forwarded_ip) {
                                        ip = None;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    if let Some(last) = self.headers.get_all(a_name).iter().next_back() {
                        let pieces = last
                            .to_str()
                            .unwrap()
                            .split(",")
                            .map(|e| e.to_string())
                            .collect::<Vec<String>>();

                        let mut values = pieces
                            .iter()
                            .flat_map(|entry| IpAddr::from_str(entry.trim()))
                            .collect::<VecDeque<IpAddr>>();
                        if pieces.len() == values.len() {
                            ip = values.pop_front();
                            break;
                        }
                    }
                }
            }
        }
        ip
    }

    fn x_forwarded_ips(&self) -> Option<VecDeque<IpAddr>> {
        let mut ips = VecDeque::new();
        for entry in self.headers.get_all("x-forwarded-for").iter() {
            let pieces = entry
                .to_str()
                .unwrap()
                .split(",")
                .map(|e| e.to_string())
                .collect::<Vec<String>>();

            let values = pieces
                .iter()
                .flat_map(|entry| IpAddr::from_str(entry.trim()))
                .collect::<VecDeque<IpAddr>>();
            if pieces.len() != values.len() {
                return None;
            }

            ips.extend(values);
        }

        Some(ips)
    }
}

impl Display for HttpContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ip = if let Some(ip) = self.ip() {
            ip.to_string()
        } else {
            String::new()
        };
        let user_agent = if let Some(ua) = self.user_agent() {
            ua.to_str().unwrap().to_string()
        } else {
            String::new()
        };
        write!(f, "{ip}::{user_agent}")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustedIp {
    All,
    Net(ipnet::IpNet),
    Ip(IpAddr),
}

impl FromStr for TrustedIp {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" {
            return Ok(Self::All);
        }

        if s.contains('/') {
            if let Ok(net) = ipnet::IpNet::from_str(s) {
                return Ok(Self::Net(net));
            }
        } else if let Ok(ip) = IpAddr::from_str(s) {
            return Ok(Self::Ip(ip));
        }
        Err(String::from("could not parse IP"))
    }
}

impl TrustedIp {
    pub fn passes(&self, other: &IpAddr) -> bool {
        match self {
            Self::All => true,
            Self::Net(net) => net.contains(other),
            Self::Ip(ip) => ip == other,
        }
    }

    pub fn form_collection<S, I>(value: S) -> Vec<Self>
    where
        S: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        value
            .into_iter()
            .flat_map(|e| Self::from_str(e.as_ref()))
            .collect::<Vec<TrustedIp>>()
    }
}

#[cfg(test)]
mod test {
    use axum::{body::Body, http::Request};

    use super::*;

    #[tokio::test]
    async fn test_path() {
        let mut req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            .body(Body::empty())
            .unwrap();
        let ctx = HttpContext::from_request(&mut req).await;

        assert_eq!(ctx.path(), "/path1/path2");
        assert_eq!(
            ctx.full_path(),
            "https://yahoo.com/path1/path2?query1=value1&query2=value2"
        );
    }

    #[tokio::test]
    async fn test_query() {
        let mut req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            .body(Body::empty())
            .unwrap();
        let ctx = HttpContext::from_request(&mut req).await;
        assert_eq!(ctx.query(), Some("query1=value1&query2=value2".to_string()));
        assert_eq!(ctx.query_as_map().is_some(), true);
        assert_eq!(ctx.query_as_map().unwrap().len(), 2)
    }

    #[tokio::test]
    async fn test_host() {
        let mut req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            .body(Body::empty())
            .unwrap();
        let ctx = HttpContext::from_request(&mut req).await;
        assert_eq!(ctx.host().is_some(), true);
        assert_eq!(ctx.host().unwrap(), "yahoo.com");
    }

    #[tokio::test]
    async fn test_ip_from_forwarded_for1() {
        let req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            .header("X-FORWARDED-FOR", "192.168.0.100")
            .header("X-FORWARDED-FOR", "192.168.0.8")
            // .header("X-FORWARDED-FOR", "192.168.0.5, 192.168.0.44, 192.168.3.6")
            .body(Body::empty())
            .unwrap();

        let trusted = TrustedIp::form_collection(["192.168.0.5/24"]);
        let ctx = HttpContext::new(&req, &["x-forwarded-for"], &trusted).await;

        let result = ctx.ip().unwrap();
        let ip = IpAddr::from_str("192.168.0.100").unwrap();
        assert_eq!(result, ip);
    }

    #[tokio::test]
    async fn test_ip_from_forwarded_for2() {
        let req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            .header("X-FORWARDED-FOR", "192.168.0.100")
            .header("X-FORWARDED-FOR", "192.168.0.8")
            // .header("X-FORWARDED-FOR", "192.168.0.5, 192.168.0.44, 192.168.3.6")
            .body(Body::empty())
            .unwrap();

        let trusted = TrustedIp::form_collection(["192.168.1.5/24"]);
        let ctx = HttpContext::new(&req, &["x-forwarded-for"], &trusted).await;

        let result = ctx.ip();
        assert_eq!(result.is_none(), true);
    }

    #[tokio::test]
    async fn test_raw_path() {
        let req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            .header("X-FORWARDED-FOR", "192.168.0.100")
            .header("X-FORWARDED-FOR", "192.168.0.8")
            // .header("X-FORWARDED-FOR", "192.168.0.5, 192.168.0.44, 192.168.3.6")
            .body(Body::empty())
            .unwrap();

        let trusted = TrustedIp::form_collection(["192.168.1.5/24"]);
        let ctx = HttpContext::new(&req, &["x-forwarded-for"], &trusted).await;

        println!("{:?}", ctx.get_a_path_by::<String>("path1"));
    }

    #[test]
    fn test_trusted_ip() {
        let ip1 = IpAddr::from_str("192.168.0.2").unwrap();
        let ip2 = IpAddr::from_str("192.168.0.10").unwrap();

        let trusted1 = TrustedIp::from_str("192.168.0.2").unwrap();
        let trusted2 = TrustedIp::from_str("192.168.0.2/24").unwrap();

        assert_eq!(trusted1.passes(&ip1), true);
        assert_eq!(trusted1.passes(&ip2), false);

        assert_eq!(trusted2.passes(&ip2), true);
        assert_eq!(trusted2.passes(&ip2), true);
    }
}
