//! Common Expression Language (CEL)
//!
//! Link: https://cel.dev
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use cel::{Context as CelCtx, ExecutionError as CelExecutionError, Program, Value};

use crate::prelude::{
    Context, ContextResourceManager, Observable, global_context, observable::cel::CelCoreVariable,
};

/// An Observable that allows other crates to add function, variable to global context
pub type CelContext<'a> = CelCtx<'a>;

pub use cel;

#[async_trait::async_trait]
impl<'a> Observable for CelContext<'a> {}

#[derive(Debug)]
pub enum CelManagerError {
    ProgramNotFound,
    ExecutionError(CelExecutionError),
}

#[derive(Default, Clone)]
pub struct CommonExpressionSandbox {
    collection: Arc<RwLock<HashMap<String, Program>>>,
}

impl CommonExpressionSandbox {
    pub async fn new() -> Self {
        let result = busybody::helpers::get_type::<CommonExpressionSandbox>().await;
        if result.is_none() {
            Self::setup().await;
            return busybody::helpers::get_type::<CommonExpressionSandbox>()
                .await
                .unwrap_or_default();
        }

        result.unwrap() // NOTE: Already checked for "Some"
    }

    /// Register a program
    /// This function does not check if the program already exist
    pub fn set_program(&self, name: &str, source: &str) -> &Self {
        match Program::compile(source) {
            Ok(program) => match self.collection.write() {
                Ok(mut w_lock) => {
                    w_lock.insert(name.to_string(), program);
                }
                Err(e) => {
                    tracing::error!("{}", e);
                }
            },
            Err(e) => {
                tracing::error!("error parsing program: {}", e);
            }
        }
        self
    }

    /// Update a existing program.
    pub fn put_program(&self, name: &str, source: &str) -> &Self {
        match self.collection.read() {
            Ok(r_lock) => {
                if !r_lock.contains_key(name) {
                    return self;
                }
            }
            Err(e) => {
                tracing::error!("{}", e);
                return self;
            }
        }
        self.set_program(name, source)
    }

    /// Register a program only if the program does not exist
    pub fn add_program(&self, name: &str, source: &str) -> &Self {
        match self.collection.read() {
            Ok(r_lock) => {
                if r_lock.contains_key(name) {
                    return self;
                }
            }
            Err(e) => {
                tracing::error!("{}", e);
                return self;
            }
        }

        self.set_program(name, source)
    }

    pub async fn execute(&self, context: &Context, name: &str) -> Result<Value, CelManagerError> {
        self.execute_with(context, name, |_| {}).await
    }

    pub async fn execute_with<F>(
        &self,
        context: &Context,
        name: &str,
        callback: F,
    ) -> Result<Value, CelManagerError>
    where
        F: Fn(&mut CelCtx<'_>) -> (),
    {
        let ctx = context.clone();
        context
            .container_ref()
            .resolver(move |_| {
                let c = ctx.clone();
                async move {
                    let mut cel_ctx = CelContext::default();
                    let core_variable = CelCoreVariable::default().notify(&c).await;
                    if let Err(e) = core_variable.merge_into_context(&mut cel_ctx) {
                        tracing::error!("could not merge CEL core variables. {}", e);
                    }

                    busybody::Service::new(cel_ctx.notify(&c).await)
                }
            })
            .await;

        if let Some(parent) = context.container_ref().get::<CelContext>().await {
            let mut local_ctx = parent.new_inner_scope();
            callback(&mut local_ctx);

            match self.collection.read() {
                Ok(r_lock) => {
                    if let Some(program) = r_lock.get(name) {
                        let result = program.execute(&local_ctx);
                        return result.map_err(|e| {
                            //
                            CelManagerError::ExecutionError(e)
                        });
                    }
                }
                Err(e) => {
                    tracing::error!("{}", e);
                }
            }
        }

        Err(CelManagerError::ProgramNotFound)
    }

    pub async fn global_context() -> busybody::Service<CelContext<'static>> {
        _ = Self::new().await;
        if let Some(parent) = busybody::helpers::get_service::<CelContext>().await {
            return parent;
        }
        tracing::error!("could not pluck cel global context for busybody");
        busybody::Service::new(CelContext::default())
    }

    pub(crate) async fn setup() {
        busybody::helpers::resolver_once(|_| async move {
            let manager = CommonExpressionSandbox::default();
            let a_ctx = global_context().await; // CEL global context must be attached to the application global context
            manager.notify(&a_ctx).await
        })
        .await;
    }

    pub(crate) async fn register_as_resource() {
        ContextResourceManager::register(
            |_| async { Ok(("common-expression-language", 0).into()) },
            |_| async { Ok(Self::new().await) },
            |_| async {
                //
            },
        )
        .await;
    }
}

#[async_trait::async_trait]
impl Observable for CommonExpressionSandbox {}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn test_program_and_context_manager() {
        let global_ctx = global_context().await;
        CelContext::subscribe(|mut cel_ctx, _| async move {
            cel_ctx.add_function("get_back", |x: i64| x);
            cel_ctx
                .add_variable("environment", "dev")
                .expect("could not add variable");
            cel_ctx
        })
        .await;

        CommonExpressionSandbox::subscribe(|col, _| async move {
            col.set_program("foo", "environment");
            col
        })
        .await;

        let app_manager = CommonExpressionSandbox::new().await;

        let result = app_manager
            .execute_with(&global_ctx, "foo", |ctx| {
                ctx.add_variable("environment", "prod").unwrap();
            })
            .await;

        assert!(result.is_ok());
        if let Ok(Value::String(value)) = result {
            assert_eq!(value.as_str(), "prod");
        } else {
            panic!("Expected a string value");
        }

        let result = app_manager.execute(&global_ctx, "foo").await;
        assert!(result.is_ok());
        if let Ok(Value::String(value)) = result {
            assert_eq!(value.as_str(), "dev");
        } else {
            panic!("Expected a string value");
        }

        app_manager.add_program("foo2", "true == false");

        let result = app_manager.execute(&global_ctx, "foo2").await;
        if let Ok(Value::Bool(value)) = result {
            assert!(value == false);
        } else {
            panic!("Expected a boolean value");
        }

        let result = app_manager.execute(&global_ctx, "does-not-exist").await;
        assert!(result.is_err());
    }
}
