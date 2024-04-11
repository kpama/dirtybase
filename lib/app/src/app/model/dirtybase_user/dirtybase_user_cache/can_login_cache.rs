use dirtybase_cache::CacheManager;

use crate::app::{
    helper::sha256,
    model::dirtybase_user::dirtybase_user_helpers::authentication_error_status::AuthenticationErrorStatus,
};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CanLoginCachedData {
    pub allow: bool,
    pub error: AuthenticationErrorStatus,
}

impl CanLoginCachedData {
    pub async fn store(
        cache_manager: &CacheManager,
        user_email: &str,
        allow: bool,
        error: AuthenticationErrorStatus,
    ) {
        let data = Self { allow, error };
        let id = Self::generate_id(user_email);

        cache_manager.put(&id, &data, None).await;
    }

    pub async fn fetch(cache_manager: &CacheManager, user_email: &str) -> Option<Self> {
        let id = Self::generate_id(user_email);
        cache_manager.get::<Self>(&id).await
    }

    pub async fn clear(cache_manager: &CacheManager, user_email: &str) {
        let id = Self::generate_id(user_email);
        cache_manager.forget(&id).await;
    }

    fn generate_id(email: &str) -> String {
        sha256::hash_str(email)
    }
}
