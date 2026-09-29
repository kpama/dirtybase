use std::{collections::BTreeMap, future::Future, sync::Arc};

use dirtybase_db::field_values::FieldValue;
use futures::future::BoxFuture;

use crate::{
    action::Action, changeset::Changeset, context::ActionContext, data_layer::DataLayer,
    error::Result, record::Record, resource::ResourceDef,
};

/// Everything a generic action's implementation is given.
///
/// A generic action is a domain behaviour that is not a simple read or write:
/// Ash's `action :analyze_text, :map`. It receives the resolved arguments, the
/// current record if there is one, and the ambient context, and returns a value
/// of its own choosing.
#[derive(Clone)]
pub struct GenericInput {
    resource: Arc<ResourceDef>,
    action: Arc<Action>,
    arguments: BTreeMap<String, FieldValue>,
    context: Arc<dyn ActionContext>,
    data_layer: Arc<dyn DataLayer>,
    data: Record,
    attributes: BTreeMap<String, FieldValue>,
}

impl std::fmt::Debug for GenericInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericInput")
            .field("resource", &self.resource.name())
            .field("action", &self.action.name())
            .field("arguments", &self.arguments)
            .finish_non_exhaustive()
    }
}

impl GenericInput {
    pub fn new(
        resource: Arc<ResourceDef>,
        action: Arc<Action>,
        arguments: BTreeMap<String, FieldValue>,
        context: Arc<dyn ActionContext>,
        data_layer: Arc<dyn DataLayer>,
    ) -> Self {
        Self {
            resource,
            action,
            arguments,
            context,
            data_layer,
            data: Record::new(),
            attributes: BTreeMap::new(),
        }
    }

    /// The data layer this action reads and writes through.
    pub fn data_layer(&self) -> &Arc<dyn DataLayer> {
        &self.data_layer
    }

    pub fn with_data(mut self, data: Record) -> Self {
        self.data = data;
        self
    }

    pub fn with_attributes(mut self, attributes: BTreeMap<String, FieldValue>) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn resource(&self) -> &Arc<ResourceDef> {
        &self.resource
    }

    pub fn action(&self) -> &Arc<Action> {
        &self.action
    }

    pub fn argument(&self, name: &str) -> Option<FieldValue> {
        self.arguments.get(name).cloned()
    }

    pub fn arguments(&self) -> &BTreeMap<String, FieldValue> {
        &self.arguments
    }

    pub fn context(&self) -> &Arc<dyn ActionContext> {
        &self.context
    }

    pub fn actor(&self) -> Option<crate::context::Actor> {
        self.context.actor().cloned()
    }

    pub fn data(&self) -> &Record {
        &self.data
    }

    pub fn attributes(&self) -> &BTreeMap<String, FieldValue> {
        &self.attributes
    }

    /// Run another action from inside this one.
    ///
    /// This is what lets a generic action be composed out of other actions
    /// rather than reimplementing reads and writes. The ambient context is
    /// passed down, so the actor and tenant of the outer call are visible to the
    /// nested action.
    pub async fn run(&self, changeset: Changeset) -> Result<Record> {
        crate::domain::run_changeset_with(&self.data_layer, changeset, self.context.as_ref()).await
    }

    /// Convenience for the very common "read one by id" case.
    pub async fn read_one_by_id(&self, id: &str) -> Result<Record> {
        let mut query = crate::query::Query::new(Arc::clone(&self.resource), self.action.clone());
        query.filter_eq("id", id);
        self.data_layer
            .read_one(Arc::clone(&self.resource), &query)
            .await?
            .ok_or_else(|| {
                crate::error::Error::changeset(format!(
                    "no `{}` record with id {id}",
                    self.resource.name()
                ))
                .into()
            })
    }
}

/// The signature of a generic action's implementation.
pub type GenericRunner =
    Arc<dyn Fn(GenericInput) -> BoxFuture<'static, Result<FieldValue>> + Send + Sync>;

/// Wrap a closure as a generic action runner.
pub fn runner<F, Fut>(f: F) -> GenericRunner
where
    F: Fn(GenericInput) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<FieldValue>> + Send + 'static,
{
    Arc::new(move |input| Box::pin(f(input)))
}
