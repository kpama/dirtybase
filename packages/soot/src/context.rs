use std::{collections::BTreeMap, sync::Arc};

use dirtybase_db::field_values::FieldValue;

/// The authenticated subject performing an action.
///
/// Ash calls this `%User{}`. Here it is deliberately loose: an actor is a bag
/// of attribute values keyed by name, so a soot resource can be used as the
/// actor without any coupling between the two.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Actor {
    primary_key: Option<FieldValue>,
    attributes: BTreeMap<String, FieldValue>,
}

impl Actor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_id(id: impl Into<FieldValue>) -> Self {
        Self {
            primary_key: Some(id.into()),
            attributes: BTreeMap::new(),
        }
    }

    pub fn put(mut self, name: &str, value: impl Into<FieldValue>) -> Self {
        self.attributes.insert(name.to_string(), value.into());
        self
    }

    pub fn get(&self, name: &str) -> Option<FieldValue> {
        self.attributes.get(name).cloned()
    }

    pub fn id(&self) -> Option<FieldValue> {
        self.primary_key.clone()
    }

    pub fn set_id(mut self, id: impl Into<FieldValue>) -> Self {
        self.primary_key = Some(id.into());
        self
    }

    pub fn attributes(&self) -> &BTreeMap<String, FieldValue> {
        &self.attributes
    }

    /// Resolve the value of an `actor(:name)` expression used inside a change
    /// or a query filter.
    pub fn resolve(&self, name: &str) -> Option<FieldValue> {
        match name {
            "__id__" | "__primary_key__" => self.primary_key.clone(),
            _ => self.get(name),
        }
    }
}

/// Everything a change, validation or preparation needs that is not part of the
/// changeset itself.
///
/// Callers supply this alongside a changeset; the default
/// [`DefaultActionContext`] carries just an actor and a free form context map,
/// matching Ash's `context` argument.
///
/// This trait is used as `dyn ActionContext` throughout the engine, so every
/// method has to stay object safe. That is why `with_value` takes a
/// [`FieldValue`] rather than `impl Into<FieldValue>`: a generic method would
/// make the trait impossible to use behind a trait object.
pub trait ActionContext: Send + Sync {
    fn actor(&self) -> Option<&Actor>;
    fn tenant(&self) -> Option<FieldValue>;

    /// A free form value the caller threaded through for the action to read.
    fn get(&self, key: &str) -> Option<FieldValue>;

    /// Every context value, for passing down to nested actions.
    fn context(&self) -> BTreeMap<String, FieldValue> {
        BTreeMap::new()
    }

    /// A copy of this context with `key` set. Nested actions receive the result
    /// so that values survive across a change that calls another action.
    fn with_value(&self, key: &str, value: FieldValue) -> Box<dyn ActionContext>;

    /// An owned handle to this context.
    ///
    /// A generic action runs after the caller's borrow has ended, so it takes
    /// ownership of the context. Handing the handle to a nested action is what
    /// keeps the actor and tenant of the outer call visible all the way down.
    fn handle(&self) -> Arc<dyn ActionContext>;
}

#[derive(Clone, Default)]
pub struct DefaultActionContext {
    actor: Option<Actor>,
    tenant: Option<FieldValue>,
    values: BTreeMap<String, FieldValue>,
}

impl DefaultActionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_actor(actor: Actor) -> Self {
        Self {
            actor: Some(actor),
            ..Default::default()
        }
    }

    pub fn with_tenant(tenant: impl Into<FieldValue>) -> Self {
        Self {
            tenant: Some(tenant.into()),
            ..Default::default()
        }
    }

    /// Both at once, which is what a request authenticated inside a tenant has.
    /// The two builders above each start from the default, so a caller holding
    /// both an actor and a tenant needs this rather than chaining them.
    pub fn with_actor_and_tenant(actor: Actor, tenant: impl Into<FieldValue>) -> Self {
        Self {
            actor: Some(actor),
            tenant: Some(tenant.into()),
            ..Default::default()
        }
    }

    pub fn with_value(mut self, key: &str, value: impl Into<FieldValue>) -> Self {
        self.values.insert(key.to_string(), value.into());
        self
    }
}

impl ActionContext for DefaultActionContext {
    fn actor(&self) -> Option<&Actor> {
        self.actor.as_ref()
    }

    fn tenant(&self) -> Option<FieldValue> {
        self.tenant.clone()
    }

    fn get(&self, key: &str) -> Option<FieldValue> {
        self.values.get(key).cloned()
    }

    fn context(&self) -> BTreeMap<String, FieldValue> {
        self.values.clone()
    }

    fn with_value(&self, key: &str, value: FieldValue) -> Box<dyn ActionContext> {
        let mut next = self.clone();
        next.values.insert(key.to_string(), value);
        Box::new(next)
    }

    fn handle(&self) -> Arc<dyn ActionContext> {
        Arc::new(self.clone())
    }
}
