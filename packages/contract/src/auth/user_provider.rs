use std::sync::Arc;

pub struct UserProviderManager;

#[async_trait::async_trait]
pub trait UserProviderTrait: Send + Sync {
    /// Try getting a user by their username
    async fn by_username(&self, username: &str) -> String;

    /// Try getting a user by their email
    async fn by_email(&self, email: &str) -> String;

    /// Try getting a user by their id
    /// This is not the database internal ID
    async fn by_id(&self, id: &str) -> String;
}

#[async_trait::async_trait]
impl UserProviderTrait for UserProviderManager {
    async fn by_username(&self, id: &str) -> String {
        println!("using the default user provider service");
        if id == "123" {
            return 200.to_string();
        }
        String::new()
    }

    async fn by_email(&self, _email: &str) -> String {
        String::new()
    }

    async fn by_id(&self, _id: &str) -> String {
        String::new()
    }
}

#[derive(Clone)]
pub struct UserProviderService(Arc<Box<dyn UserProviderTrait + 'static>>);

impl UserProviderService {
    pub fn new<T: UserProviderTrait + 'static>(inner: T) -> Self {
        Self(Arc::new(Box::new(inner)))
    }
}

#[async_trait::async_trait]
impl UserProviderTrait for UserProviderService {
    async fn by_username(&self, id: &str) -> String {
        self.0.by_username(id).await
    }

    async fn by_email(&self, email: &str) -> String {
        self.0.by_email(email).await
    }

    async fn by_id(&self, id: &str) -> String {
        self.0.by_id(id).await
    }
}
