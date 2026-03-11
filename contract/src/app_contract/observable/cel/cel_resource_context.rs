use crate::prelude::Observable;

///! An Observable that is use to build the resource object for the CEL sandbox

#[derive(Debug, Default, serde::Serialize)]
pub struct CelResourceContext(serde_json::Map<String, serde_json::Value>);

impl CelResourceContext {
    pub fn add<T: serde::Serialize>(&mut self, key: &str, value: T) -> &mut Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.0.insert(key.to_string(), v);
        }
        self
    }

    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.0.remove(key)
    }

    pub fn new<T: serde::Serialize>(resource: T) -> Self {
        let resource = if let Ok(v) = serde_json::to_value(resource)
            && v.is_object()
        {
            v.as_object().cloned().unwrap()
        } else {
            serde_json::value::Map::default()
        };

        Self(resource)
    }
}

#[async_trait::async_trait]
impl Observable for CelResourceContext {}

#[cfg(test)]
mod test {
    use serde::Serialize;

    use crate::prelude::global_context;

    use super::*;

    #[tokio::test]
    async fn test() {
        #[derive(Serialize)]
        struct Product {
            id: String,
            name: String,
        }

        CelResourceContext::subscribe(|mut resource, _| async move {
            resource.add("owner", 544);
            resource
        })
        .await;

        let product = Product {
            id: "abc".to_string(),
            name: "product ABC".to_string(),
        };
        let ctx = CelResourceContext::new(&product)
            .notify(&global_context().await)
            .await;

        println!("{:?}", serde_json::to_string_pretty(&ctx));
    }
}
