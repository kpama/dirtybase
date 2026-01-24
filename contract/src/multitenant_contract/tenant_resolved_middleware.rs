///! Tenant Resolved middleware
///!  When a tenant has been resolved, this middlware is called passing the
///!  tenant to all registered middlewares
use std::sync::Arc;

use crate::multitenant_contract::model::Tenant;

type TenantMiddlewareType = simple_middleware::Manager<Tenant, Result<Tenant, anyhow::Error>>;

pub struct TenantResolvedMiddleware;

impl TenantResolvedMiddleware {
    /// Get the middleware manager instance
    ///
    /// Use the return middleware manager to registered your middleware
    pub async fn get() -> Arc<TenantMiddlewareType> {
        match busybody::helpers::service_container().get().await {
            Some(m) => m,
            None => {
                let manager =
                    simple_middleware::Manager::<Tenant, Result<Tenant, anyhow::Error>>::last(
                        |t, _| async move { Ok(t) },
                    )
                    .await;

                busybody::helpers::service_container()
                    .set(manager)
                    .await
                    .get()
                    .await
                    .unwrap() // NOTE: Just added the instance, should never fail
            }
        }
    }
}
