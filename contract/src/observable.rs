use crate::prelude::Context;

/// Type implementing this trait are dispatchable
///
/// You can subscribe to be notified when the instance is dispatched
/// This is allows you to hook into various logic flow of the application.
///
/// However, this is not a "event" that is dispatched and forget. Some of
/// the dispatched types are not serializable.
#[async_trait::async_trait]
pub trait Observable {
    /// Register an observer that will be notified
    async fn subscribe<F, R>(mut handler: F)
    where
        F: FnMut(Self, Context) -> R + Send + 'static,
        R: Future<Output = Self> + Send + 'static,
        Self: Sized + 'static,
    {
        let manager = if let Some(manager) =
            busybody::helpers::get_service::<simple_middleware::Manager<(Self, Context), Self>>()
                .await
        {
            manager
        } else {
            let manager = simple_middleware::Manager::<(Self, Context), Self>::last(
                |(data, _), _| async move { data },
            )
            .await;

            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap()
        };

        manager
            .next(move |(data, ctx), next| {
                let result = handler(data, ctx.clone());
                async move { next.call((result.await, ctx)).await }
            })
            .await;
    }

    /// Notify the observers
    async fn notify(self, context: &Context) -> Self
    where
        Self: Sized + 'static,
    {
        let manager = if let Some(manager) =
            busybody::helpers::get_service::<simple_middleware::Manager<(Self, Context), Self>>()
                .await
        {
            manager
        } else {
            let manager = simple_middleware::Manager::<(Self, Context), Self>::last(
                |(data, _), _| async move { data },
            )
            .await;

            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap()
        };

        manager.send((self, context.clone())).await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_subscribing() {
        struct Saving(i32);
        impl Observable for Saving {}

        Saving::subscribe(|mut data, _| async move {
            data.0 = 44;
            data
        })
        .await;

        let instance = Saving(0).notify(&Context::new().await).await;
        assert_eq!(instance.0, 44);
    }

    #[tokio::test]
    async fn test_no_subscribers() {
        struct Saving(i32);
        impl Observable for Saving {}

        let instance = Saving(44).notify(&Context::new().await).await;
        assert_eq!(instance.0, 44);
    }
}
