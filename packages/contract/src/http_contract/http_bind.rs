use std::{future::Future, ops::Deref, sync::Arc};

use axum::extract::{rejection::PathRejection, Path};
use serde::de::DeserializeOwned;

use crate::{
    db_contract::{base::manager::Manager, field_values::FieldValue, TableEntityTrait},
    prelude::Context,
};

use super::{HttpContext, MiddlewareParam};

/// Binds a type to a URI path
///
#[derive(Clone)]
pub struct Bind<T>(pub T);

impl<T> Deref for Bind<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for Bind<T> {
    fn from(value: T) -> Self {
        Bind(value)
    }
}

impl<T: std::any::Any + Clone + Send + Sync> Bind<T> {
    /// Register a resolver for the type `T`
    ///
    /// When a request handler requires an instance of `T`,
    /// this resolver will be called.
    pub async fn resolver<F, Fut>(callback: F)
    where
        F: Clone + Fn(ModelBindResolver) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<Bind<T>>> + Send + 'static,
    {
        ModelBindResolver::register(std::any::type_name::<T>(), callback).await
    }

    /// Register an alias for the type `T`
    ///
    /// When a URI is requested that has this alias, the resolver will be called
    pub async fn alias(alias: &str) {
        ModelBindResolver::alias::<T>(alias).await;
    }
}

/// Implement a general resolver for types that implements `TableEntityTrait`
impl<T: TableEntityTrait + 'static> Bind<T> {
    /// Binds a URI `path` to a table `column`
    ///
    /// If the table column is None, the `id` column will be used
    pub async fn from_to<F: DeserializeOwned + Into<FieldValue> + Send + Sync + 'static>(
        from: &str,
        to: Option<&str>,
    ) {
        let field_name = Arc::new(to.unwrap_or("id").to_string());
        let path_name = Arc::new(from.to_string());

        ModelBindResolver::register(std::any::type_name::<T>(), move |resolver| {
            let name = field_name.clone();
            let path = path_name.clone();
            async move {
                if let Some(field_value) = resolver.http_context_ref().get_a_path_by::<F>(&path) {
                    if let Ok(manager) = resolver.context_ref().get::<Manager>().await {
                        if let Ok(Some(value)) = manager
                            .select_from::<T>(|query| {
                                query.eq(name, field_value);
                            })
                            .fetch_one_to::<T>()
                            .await
                        {
                            return Some(Bind(value));
                        }
                    }
                }

                None
            }
        })
        .await
    }
}

/// An instance of this struct is passed to your registered resolver
#[derive(Clone)]
pub struct ModelBindResolver {
    pub(crate) context: Context,
    pub(crate) http_ctx: HttpContext,
    pub(crate) args: MiddlewareParam,
}

impl ModelBindResolver {
    pub(crate) async fn new(context: Context, args: Option<MiddlewareParam>) -> Self {
        Self {
            http_ctx: context.get::<HttpContext>().await.unwrap(),
            args: args.unwrap_or_default(),
            context,
        }
    }

    /// Returns the current request context
    pub fn context(&self) -> Context {
        self.context.clone()
    }

    /// The middleware parameter instance
    pub fn args(&self) -> &MiddlewareParam {
        &self.args
    }

    /// Returns a reference of the request context instead of a "clone"
    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    /// Returns a references of the current http context
    pub fn http_context_ref(&self) -> &HttpContext {
        &self.http_ctx
    }

    pub async fn get_path<T>(&mut self) -> Result<Path<T>, PathRejection>
    where
        T: DeserializeOwned + Send,
    {
        self.http_ctx.get_path().await
    }

    pub(crate) async fn bind<T: 'static>(self) -> Result<Option<Bind<T>>, anyhow::Error> {
        Self::get_middleware()
            .await
            .send((self, std::any::type_name::<T>()))
            .await
    }

    pub(crate) async fn register<T: 'static, F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<Bind<T>>> + Send + 'static,
    {
        let resolvers = Self::get_middleware().await;
        let arc_name = Arc::new(name.to_string());

        resolvers
            .next(move |(resolver, bind_name), next| {
                let cb = callback.clone();
                let name = arc_name.clone();
                Box::pin(async move {
                    if bind_name == name.as_str() {
                        tracing::trace!("resolving binding for '{}'", &name);
                        return Ok((cb)(resolver).await);
                    }
                    next.call((resolver, bind_name)).await
                })
            })
            .await;
    }

    pub(crate) async fn inject_alias(self, alias: &str) -> Result<bool, anyhow::Error> {
        Self::get_alias_middleware()
            .await
            .send((self, alias.to_string()))
            .await
    }

    pub(crate) async fn inject_all_bindings(self) -> Result<bool, anyhow::Error> {
        for alias in self.http_ctx.get_path_names() {
            if let Ok(val) = Self::get_alias_middleware()
                .await
                .send((self.clone(), alias.to_string()))
                .await
            {
                if !val {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    pub(crate) async fn get_middleware<T: 'static>(
    ) -> Arc<simple_middleware::Manager<(Self, &'static str), Result<Option<Bind<T>>, anyhow::Error>>>
    {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<
                (Self, &'static str),
                Result<Option<Bind<T>>, anyhow::Error>,
            >::last(|(_, _), _| {
                Box::pin(async move {
                    //
                    Err(anyhow::anyhow!("no handler"))
                })
            })
            .await;
            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap() // should never failed as we just registered the instance
        }
    }

    pub(crate) async fn alias<T: Clone + Send + Sync + 'static>(alias: &str) {
        let alias_name = Arc::new(alias.to_string());
        Self::get_alias_middleware()
            .await
            .next(move |(resolver, name), next| {
                let alias = alias_name.clone();
                Box::pin(async move {
                    //
                    if name == *alias {
                        //
                        tracing::trace!(
                            "resolving binding for '{}' via alias '{}'",
                            std::any::type_name::<T>(),
                            &name
                        );
                        if let Ok(Some(Bind(value))) = resolver.clone().bind::<T>().await {
                            resolver.context_ref().set(value).await;
                            return Ok(true);
                        }
                        return Ok(false);
                    }
                    next.call((resolver, name)).await
                })
            })
            .await;
    }

    pub(crate) async fn get_alias_middleware(
    ) -> Arc<simple_middleware::Manager<(Self, String), Result<bool, anyhow::Error>>> {
        if let Some(m) = busybody::helpers::service_container().get().await {
            m
        } else {
            let manager =
                simple_middleware::Manager::<(Self, String), Result<bool, anyhow::Error>>::last(
                    |(_, _), _| {
                        Box::pin(async move {
                            //
                            Err(anyhow::anyhow!("no handler for this alias"))
                        })
                    },
                )
                .await;
            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap() // should never failed as we just registered the instance
        }
    }
}
