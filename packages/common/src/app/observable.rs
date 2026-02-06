#[async_trait::async_trait]
pub trait Observable {
    async fn subscribe<F, R>(mut handler: F)
    where
        F: FnMut(Self) -> R + Send + 'static,
        R: Future<Output = Self> + Send + 'static,
        Self: Sized + 'static,
    {
        let manager = if let Some(manager) =
            busybody::helpers::get_service::<simple_middleware::Manager<Self, Self>>().await
        {
            manager
        } else {
            let manager =
                simple_middleware::Manager::<Self, Self>::last(|data, _| async move { data }).await;

            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap()
        };

        manager
            .next(move |data, next| {
                let result = handler(data);
                async move { next.call(result.await).await }
            })
            .await;
    }

    async fn dispatch(self) -> Self
    where
        Self: Sized + 'static,
    {
        let manager = if let Some(manager) =
            busybody::helpers::get_service::<simple_middleware::Manager<Self, Self>>().await
        {
            manager
        } else {
            let manager =
                simple_middleware::Manager::<Self, Self>::last(|data, _| async move { data }).await;

            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap()
        };

        manager.send(self).await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_subscribing() {
        struct Saving(i32);
        impl Observable for Saving {}

        Saving::subscribe(|mut data| async move {
            data.0 = 44;
            data
        })
        .await;

        let instance = Saving(0).dispatch().await;
        assert_eq!(instance.0, 44);
    }

    #[tokio::test]
    async fn test_no_subscribers() {
        struct Saving(i32);
        impl Observable for Saving {}

        let instance = Saving(44).dispatch().await;
        assert_eq!(instance.0, 44);
    }
}
