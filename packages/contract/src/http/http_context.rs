use std::{collections::HashMap, str::FromStr, sync::OnceLock};

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, Uri},
};
use tokio::io::split;

use crate::db::base::query;

#[derive(Clone)]
pub struct HttpContext {
    uri: Uri,
    headers: HeaderMap<HeaderValue>,
    client_id: OnceLock<Option<String>>,
}

impl HttpContext {
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    pub fn full_path(&self) -> String {
        let mut full_path = String::new();

        // http:// or https://
        if let Some(scheme) = self.uri.scheme_str() {
            full_path.push_str(&format!("{}://", scheme));
        }

        // foo.com or 127.0.0.1
        if let Some(host) = self.uri.host() {
            full_path.push_str(&host);
        }

        // /home or /home?a=1&b=2
        if let Some(path_n_query) = self.uri.path_and_query() {
            full_path.push_str(path_n_query.as_str());
        }

        full_path
    }

    pub fn query(&self) -> Option<String> {
        match self.uri.query() {
            Some(q) => Some(q.to_string()),
            None => None,
        }
    }

    pub fn host(&self) -> Option<&str> {
        self.uri.host()
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
            if map.len() > 0 {
                return Some(map);
            }
        }

        None
    }

    pub fn ip_from_headers<H: IntoIterator<Item = I>, P: IntoIterator<Item = I>, I: ToString>(
        &self,
        trusted_headers: H,
        trusted_ips: P,
    ) {
        // if self.client_id.get_or_init(||);
        let mut accept_all = false;
        let names = trusted_headers
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>();
        let ips_as_string = trusted_ips
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>();
        accept_all = ips_as_string.contains(&"*".to_string());

        'outer: for a_name in &names {
            if let Some(last_entry) = self.headers.get_all(a_name).iter().last() {
                let mut values = last_entry
                    .to_str()
                    .unwrap()
                    .split(",")
                    .map(|e| {
                        //
                        let s = e.to_string();
                        println!("ip entry in header: {}", &s);
                        s
                    })
                    .map(|entry| ipnet::IpNet::from_str(entry.trim()))
                    .filter(|e| {
                        if let Err(error) = &e {
                            println!("error parsing string to IP address: {}", error);
                            false
                        } else {
                            true
                        }
                    })
                    .map(|e| e.unwrap())
                    .collect::<Vec<ipnet::IpNet>>();

                println!("raw values: {:?}", &values);
                // this header exist but is empty
                if values.len() == 0 {
                    continue;
                }

                // the request passed through at least one proxy
                let client_ip = values.pop();
                if values.len() > 1 && !accept_all {
                    // convert the strings to real IPs
                    let ips = ips_as_string
                        .iter()
                        .map(|e| ipnet::IpNet::from_str(e))
                        .filter(|e| e.is_ok())
                        .map(|e| e.unwrap())
                        .collect::<Vec<ipnet::IpNet>>();
                    if ips.len() != ips_as_string.len() {
                        tracing::error!("one or more trust IPs could not be person to a valid string: {:?}, {:?}", &ips_as_string, &ips);
                        println!("one or more trust IPs could not be person to a valid string: {:?}, {:?}", &ips_as_string, &ips);
                        continue 'outer;
                    }
                    // TODO: Validate each entry as a valid ip, check that this ip is in the list of trusted ip
                    for an_ip in &values {
                        if !ips.contains(an_ip) {
                            continue 'outer;
                        }
                    }
                }

                // TODO: Validate client's actual IP
                println!("header: {}, values: {:?}", &a_name, values);
                println!("header: {}, client ip: {:?}", &a_name, client_ip);
                break;
            }
        }
    }
}

impl<T> From<&Request<T>> for HttpContext {
    fn from(req: &Request<T>) -> Self {
        HttpContext {
            uri: req.uri().clone(),
            headers: req.headers().clone(),
            client_id: OnceLock::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use axum::http::Request;

    use super::*;

    #[test]
    fn test_path() {
        let req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            .body(())
            .unwrap();
        let ctx = HttpContext::from(&req);

        assert_eq!(ctx.path(), "/path1/path2");
        assert_eq!(
            ctx.full_path(),
            "https://yahoo.com/path1/path2?query1=value1&query2=value2"
        );
    }

    #[test]
    fn test_query() {
        let req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            .body(())
            .unwrap();
        let ctx = HttpContext::from(&req);
        assert_eq!(ctx.query(), Some("query1=value1&query2=value2".to_string()));
        assert_eq!(ctx.query_as_map().is_some(), true);
        assert_eq!(ctx.query_as_map().unwrap().len(), 2)
    }

    #[test]
    fn test_host() {
        let req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            .body(())
            .unwrap();
        let ctx = HttpContext::from(&req);
        assert_eq!(ctx.host().is_some(), true);
        assert_eq!(ctx.host().unwrap(), "yahoo.com");
    }

    #[test]
    fn test_ip_from_forwarded_for() {
        let req = Request::builder()
            .uri("https://yahoo.com/path1/path2?query1=value1&query2=value2")
            // .header("True-Client-IP", "10")
            .header("X-FORWARDED-FOR", "192.168.0.2, 192.168.0.5, 192.168.1.44")
            // .header("X-FORWARDED-FOR", "4, 5, 6")
            // .header("True-Client-IP", "6, 8, 11")
            // .header("X-FORWARDED-FOR", "20,21,22")
            .body(())
            .unwrap();
        let ctx = HttpContext::from(&req);

        let result = req
            .headers()
            .get_all("x-forwarded-for")
            .iter()
            .last()
            .map(|v| {
                //
                v.to_str()
                    .unwrap()
                    .split(",")
                    .map(|entry| entry.to_string())
                    .collect::<Vec<String>>()
            });
        println!("x-forwarded-for header: {:?}", result);
        println!("headers: {:?}", req.headers());
        ctx.ip_from_headers(
            &["x-forwarded-for", "X-Real-IP", "True-Client-IP"],
            &["192.168.0.2", "192.168.0.5"],
        );
    }
}
