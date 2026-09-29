use std::collections::BTreeMap;

use dirtybase_db::field_values::FieldValue;

use crate::error::Error;

/// The five kinds of action. Matches Ash's action types exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    /// Returns records. Filters and sorts live in a [`crate::query::Query`].
    Read,
    /// Returns a newly inserted record.
    Create,
    /// Returns the updated record.
    Update,
    /// Returns the destroyed record.
    Destroy,
    /// Runs arbitrary logic and returns whatever the caller declares.
    Generic,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Create => "create",
            Self::Update => "update",
            Self::Destroy => "destroy",
            Self::Generic => "generic",
        }
    }

    /// Create, update and destroy mutate state, so they carry a changeset and
    /// run in a transaction by default.
    pub fn is_changeset_based(&self) -> bool {
        matches!(self, Self::Create | Self::Update | Self::Destroy)
    }

    /// Only these can accept attributes as input.
    pub fn accepts_attributes(&self) -> bool {
        matches!(self, Self::Create | Self::Update)
    }
}

/// A value a caller passes to an action that is not an attribute.
///
/// Arguments are what let an action be parameterised, which is the mechanism
/// behind Ash's argument-based read actions: `read :top_tickets` takes a
/// `user_id` argument and a filter that references it.
#[derive(Debug, Clone)]
pub struct Argument {
    name: String,
    ty: crate::attribute::AttributeType,
    required: bool,
    private: bool,
    default: Option<FieldValue>,
    description: Option<String>,
}

impl Argument {
    pub fn new(name: &str, ty: crate::attribute::AttributeType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            required: false,
            private: false,
            default: None,
            description: None,
        }
    }

    pub fn string(name: &str) -> Self {
        Self::new(name, crate::attribute::AttributeType::String)
    }

    pub fn integer(name: &str) -> Self {
        Self::new(name, crate::attribute::AttributeType::Integer)
    }

    pub fn uuid(name: &str) -> Self {
        Self::new(name, crate::attribute::AttributeType::Uuid)
    }

    pub fn boolean(name: &str) -> Self {
        Self::new(name, crate::attribute::AttributeType::Boolean)
    }

    pub fn timestamp(name: &str) -> Self {
        Self::new(name, crate::attribute::AttributeType::Timestamp)
    }

    /// Reject the action if the argument is missing.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Hide the argument from generated interfaces. Callers must supply it
    /// through private arguments rather than ordinary input.
    pub fn private(mut self) -> Self {
        self.private = true;
        self
    }

    pub fn default(mut self, value: impl Into<FieldValue>) -> Self {
        self.default = Some(self.ty.coerce(value.into()));
        self
    }

    pub fn describe(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &crate::attribute::AttributeType {
        &self.ty
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn is_private(&self) -> bool {
        self.private
    }

    pub fn default_value(&self) -> Option<FieldValue> {
        self.default.clone()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// One named, introspectable operation on a resource.
///
/// Every action is fully described at runtime: its type, what it accepts, what
/// arguments it takes, and the ordered changes, validations and preparations
/// that run for it. That introspectability is the property the rest of Ash is
/// built on, and it is what lets the domain run any action generically.
#[derive(Clone)]
pub struct Action {
    name: String,
    ty: ActionType,
    accept: Vec<String>,
    arguments: Vec<Argument>,
    changes: Vec<crate::pipeline::PipelineEntity<dyn crate::change::Change>>,
    validations: Vec<crate::pipeline::PipelineEntity<dyn crate::validation::Validation>>,
    preparations: Vec<crate::pipeline::PipelineEntity<dyn crate::preparation::Preparation>>,
    primary: bool,
    transaction: Option<bool>,
    description: Option<String>,
    metadata: BTreeMap<String, FieldValue>,
    applied_pipelines: Vec<String>,
    run: Option<crate::generic::GenericRunner>,
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("name", &self.name)
            .field("type", &self.ty)
            .field("accept", &self.accept)
            .field("arguments", &self.arguments)
            .field("primary", &self.primary)
            .field("transaction", &self.transaction)
            .finish_non_exhaustive()
    }
}

impl Action {
    fn new(name: &str, ty: ActionType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            accept: Vec::new(),
            arguments: Vec::new(),
            changes: Vec::new(),
            validations: Vec::new(),
            preparations: Vec::new(),
            primary: false,
            transaction: None,
            description: None,
            metadata: BTreeMap::new(),
            applied_pipelines: Vec::new(),
            run: None,
        }
    }

    pub fn read(name: &str) -> Self {
        Self::new(name, ActionType::Read)
    }

    pub fn create(name: &str) -> Self {
        Self::new(name, ActionType::Create)
    }

    pub fn update(name: &str) -> Self {
        Self::new(name, ActionType::Update)
    }

    pub fn destroy(name: &str) -> Self {
        Self::new(name, ActionType::Destroy)
    }

    pub fn generic(name: &str) -> Self {
        Self::new(name, ActionType::Generic)
    }

    /// Supply the implementation of a generic action.
    ///
    /// Generic actions are how a resource exposes behaviour that is not a
    /// read or a write. Without a runner, invoking one is a framework error
    /// rather than a silent no-op.
    pub fn run(mut self, run: crate::generic::GenericRunner) -> Self {
        self.run = Some(run);
        self
    }

    /// Build a generic action from a closure.
    pub fn generic_running<F, Fut>(name: &str, run: F) -> Self
    where
        F: Fn(crate::generic::GenericInput) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<
                Output = crate::error::Result<dirtybase_db::field_values::FieldValue>,
            > + Send
            + 'static,
    {
        Self::generic(name).run(crate::generic::runner(run))
    }

    /// The attributes this action accepts as input. Replaces the previous list.
    pub fn accept(mut self, attributes: &[&str]) -> Self {
        self.accept = attributes.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Append to the accept list.
    pub fn accepting(mut self, attributes: &[&str]) -> Self {
        self.accept.extend(attributes.iter().map(|s| s.to_string()));
        self
    }

    /// Accept every writable attribute, which is Ash's `accept [:*]`.
    pub fn accept_all_writable(mut self) -> Self {
        self.accept.push("*".to_string());
        self
    }

    pub fn argument(mut self, argument: Argument) -> Self {
        self.arguments.push(argument);
        self
    }

    pub fn change<C: crate::change::Change + 'static>(mut self, change: C) -> Self {
        self.changes
            .push(crate::pipeline::PipelineEntity::always(Box::new(change)));
        self
    }

    /// Add a change that only runs when `condition` holds. Ash's
    /// `change {Slugify, attribute: :name} when present(:other)`.
    pub fn change_when<C: crate::change::Change + 'static>(
        mut self,
        change: C,
        condition: crate::pipeline::Condition,
    ) -> Self {
        self.changes.push(crate::pipeline::PipelineEntity::when(
            Box::new(change),
            condition,
        ));
        self
    }

    pub fn validate<V: crate::validation::Validation + 'static>(mut self, validation: V) -> Self {
        self.validations
            .push(crate::pipeline::PipelineEntity::always(Box::new(
                validation,
            )));
        self
    }

    pub fn validate_when<V: crate::validation::Validation + 'static>(
        mut self,
        validation: V,
        condition: crate::pipeline::Condition,
    ) -> Self {
        self.validations.push(crate::pipeline::PipelineEntity::when(
            Box::new(validation),
            condition,
        ));
        self
    }

    /// Add a preparation. Preparations only apply to read and generic actions.
    pub fn prepare<P: crate::preparation::Preparation + 'static>(mut self, preparation: P) -> Self {
        self.preparations
            .push(crate::pipeline::PipelineEntity::always(Box::new(
                preparation,
            )));
        self
    }

    /// Inline a named pipeline at this position. The pipeline's entities are
    /// spliced in exactly where the call appears, so ordering is preserved.
    pub fn pipe_through(mut self, pipeline: &crate::pipeline::Pipeline) -> Self {
        self.applied_pipelines.push(pipeline.name().to_string());
        self.apply_pipeline(pipeline);
        self
    }

    /// Append a pipeline's entities to this action. Used by
    /// [`crate::resource::ResourceDef::global_pipeline`] and by `pipe_through`.
    pub fn apply_pipeline(&mut self, pipeline: &crate::pipeline::Pipeline) {
        self.changes.extend(pipeline.changes().iter().cloned());
        self.validations
            .extend(pipeline.validations().iter().cloned());
        self.preparations
            .extend(pipeline.preparations().iter().cloned());
    }

    /// Whether this action already includes the named pipeline, used to keep
    /// `global_pipeline` from applying the same pipeline twice.
    pub fn mentions_pipeline(&self, name: &str) -> bool {
        self.applied_pipelines.iter().any(|applied| applied == name)
    }

    /// Mark this action as the one to use when none is named.
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// Force the action to run, or not run, inside a transaction.
    pub fn transaction(mut self, in_transaction: bool) -> Self {
        self.transaction = Some(in_transaction);
        self
    }

    pub fn describe(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: impl Into<FieldValue>) -> Self {
        self.metadata.insert(key.to_string(), value.into());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn action_type(&self) -> ActionType {
        self.ty
    }

    pub fn accepted(&self) -> &[String] {
        &self.accept
    }

    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    pub fn changes(&self) -> &[crate::pipeline::PipelineEntity<dyn crate::change::Change>] {
        &self.changes
    }

    pub fn validations(
        &self,
    ) -> &[crate::pipeline::PipelineEntity<dyn crate::validation::Validation>] {
        &self.validations
    }

    pub fn preparations(
        &self,
    ) -> &[crate::pipeline::PipelineEntity<dyn crate::preparation::Preparation>] {
        &self.preparations
    }

    pub fn is_primary(&self) -> bool {
        self.primary
    }

    /// The resolved transaction decision. Mutating actions default to running
    /// inside a transaction, reads and generics to not running in one.
    pub fn runs_in_transaction(&self) -> bool {
        self.transaction
            .unwrap_or_else(|| self.ty.is_changeset_based())
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn metadata(&self) -> &BTreeMap<String, FieldValue> {
        &self.metadata
    }

    pub fn runner(&self) -> Option<&crate::generic::GenericRunner> {
        self.run.as_ref()
    }

    pub fn argument_named(&self, name: &str) -> Option<&Argument> {
        self.arguments.iter().find(|a| a.name() == name)
    }

    /// Apply argument defaults and reject missing required arguments.
    pub fn resolve_arguments(
        &self,
        supplied: &BTreeMap<String, FieldValue>,
    ) -> Result<BTreeMap<String, FieldValue>, Error> {
        let mut resolved = BTreeMap::new();
        for argument in &self.arguments {
            let value = match supplied.get(argument.name()) {
                Some(value) => Some(argument.ty().coerce(value.clone())),
                None => argument.default_value(),
            };
            match value {
                Some(value) => {
                    resolved.insert(argument.name().to_string(), value);
                }
                None if argument.is_required() => {
                    return Err(Error::action_input_required(
                        argument.name(),
                        "is required but was not provided",
                    ));
                }
                None => {}
            }
        }
        // Values that do not match a declared argument are dropped, matching Ash,
        // which treats undeclared arguments as not part of this action.
        Ok(resolved)
    }
}
