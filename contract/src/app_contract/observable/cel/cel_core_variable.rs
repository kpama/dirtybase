//! CelAttribute
//!
//! Is an Observable that injects three variables into the
//! current Common Expression Language context.  Other part
//! of the application can subscribe and add more date to
//! any of these variables.
//!
//! By default, these variable will be an empty object
//!
//! |Variable | Purpose|
//! | ---     | ---    |
//! | `_resource` |  Information about current "resource" being accessed, created etc. |
//! | `_actor`    | Information about "actor" permforming the action         |
//! | `_env`      | Information about the current environemnt |
//!
use crate::prelude::{Observable, observable::cel::CelContext};

#[derive(Debug, Default, serde::Serialize)]
pub struct CelCoreVariable {
    resource: serde_json::Map<String, serde_json::Value>,
    environment: serde_json::Map<String, serde_json::Value>,
    actor: serde_json::Map<String, serde_json::Value>,
}

#[async_trait::async_trait]
impl Observable for CelCoreVariable {}

impl CelCoreVariable {
    pub fn new<R, E, A>(resource: R, environment: E, actor: A) -> Self
    where
        R: serde::Serialize,
        E: serde::Serialize,
        A: serde::Serialize,
    {
        Self {
            resource: Self::serialize(resource),
            environment: Self::serialize(environment),
            actor: Self::serialize(actor),
        }
    }

    /// Replaces the resource variable with the specified version
    pub fn set_resource<R: serde::Serialize>(&mut self, resource: R) -> &mut Self {
        self.resource = Self::serialize(resource);
        self
    }

    /// Add an entry to the "resource" variable
    /// The new value will replace an existing one
    pub fn add_to_resource<V: serde::Serialize>(
        &mut self,
        key: &str,
        value: V,
    ) -> Result<(), serde_json::Error> {
        self.resource
            .insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }

    /// Checks for a key in the resource object
    pub fn resource_has(&self, key: &str) -> bool {
        self.resource.contains_key(key)
    }

    /// Replaces the environment variable with the specified version
    pub fn set_environment<E: serde::Serialize>(&mut self, env: E) -> &mut Self {
        self.environment = Self::serialize(env);
        self
    }

    /// Add an entry to the "environment" variable
    /// The new value will replace an existing one
    pub fn add_to_environment<V: serde::Serialize>(
        &mut self,
        key: &str,
        value: V,
    ) -> Result<(), serde_json::Error> {
        self.environment
            .insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }

    /// Checks for a key in the environment
    pub fn environment_has(&self, key: &str) -> bool {
        self.environment.contains_key(key)
    }

    /// Replaces the actor variable with the specified version
    pub fn set_actor<A: serde::Serialize>(&mut self, actor: A) -> &mut Self {
        self.actor = Self::serialize(actor);
        self
    }

    /// Add an entry to the "environment" variable
    /// The new value will replace an existing one
    pub fn add_to_actor<V: serde::Serialize>(
        &mut self,
        key: &str,
        value: V,
    ) -> Result<(), serde_json::Error> {
        self.actor
            .insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }

    /// Replaces the environment variable with the specified version
    pub fn actor_has(&self, key: &str) -> bool {
        self.actor.contains_key(key)
    }

    pub(crate) fn merge_into_context(
        self,
        ctx: &mut CelContext,
    ) -> Result<(), cel::SerializationError> {
        ctx.add_variable("_resource", self.resource)?;
        ctx.add_variable("_actor", self.actor)?;
        ctx.add_variable("_env", self.environment)?;

        Ok(())
    }

    fn serialize<T: serde::Serialize>(variable: T) -> serde_json::Map<String, serde_json::Value> {
        if let Ok(v) = serde_json::to_value(variable)
            && v.is_object()
        {
            v.as_object().cloned().unwrap()
        } else {
            serde_json::value::Map::default()
        }
    }
}

#[cfg(test)]
mod test {
    use dirtybase_helper::time::current_datetime;

    use crate::prelude::{make_context, observable::cel::CommonExpressionSandbox};

    use super::*;

    #[tokio::test]
    async fn test_subscribing() {
        let app_context = make_context().await;

        // Add core variables
        CelCoreVariable::subscribe(|mut core, _| async move {
            // Resource
            core.add_to_resource("id", 2000).unwrap();
            core.add_to_resource("owner_id", 88).unwrap();
            // Env
            core.add_to_environment("client_ip", "192.168.0.44")
                .unwrap();
            core.add_to_environment("time", current_datetime()).unwrap();
            // Actor
            core.add_to_actor("id", 88).unwrap();
            core.add_to_actor("username", "foobar").unwrap();
            core.add_to_actor("role", "admin").unwrap();

            core
        })
        .await;

        let sandbox = CommonExpressionSandbox::new().await;
        sandbox.add_program("get-resource", "_resource");
        sandbox.add_program("get-env", "_env");
        sandbox.add_program("get-actor", "_actor");

        let result = sandbox.execute(&app_context, "get-resource").await;
        result.unwrap().json().unwrap();

        let result = sandbox.execute(&app_context, "get-env").await;
        result.unwrap().json().unwrap();

        let result = sandbox.execute(&app_context, "get-actor").await;
        result.unwrap().json().unwrap();
    }
}
