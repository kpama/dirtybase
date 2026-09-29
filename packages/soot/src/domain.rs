use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use dirtybase_db::field_values::FieldValue;

use crate::{
    action::{Action, ActionType},
    changeset::{Changeset, ChangesetKind},
    context::{ActionContext, DefaultActionContext},
    data_layer::DataLayer,
    error::{Error, ErrorList, Errors, Result},
    query::{INCLUDE_DELETED_FLAG, Query},
    record::Record,
    resource::ResourceDef,
};

/// A registry of resources, and the engine that runs their actions.
///
/// This is Ash's `Ash.Domain`. Its job is to be the single place that knows
/// which resources exist. Once it does, every action can be invoked by name
/// through one generic code path, and a caller never needs a typed handle to a
/// specific resource: `domain.run("Post", "publish", params)` is enough.
///
/// That is the property that makes a resource metadata-driven. The engine is
/// written once, here, and works for every resource ever registered.
#[derive(Clone, Default)]
pub struct Domain {
    resources: Vec<Arc<ResourceDef>>,
}

impl std::fmt::Debug for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Domain")
            .field(
                "resources",
                &self
                    .resources
                    .iter()
                    .map(|r| r.name())
                    .collect::<Vec<&str>>(),
            )
            .finish()
    }
}

impl Domain {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a resource from its declaration.
    pub fn add(mut self, resource: ResourceDef) -> Self {
        self.push(resource);
        self
    }

    /// Register a resource in place.
    pub fn push(&mut self, resource: ResourceDef) -> &mut Self {
        let resource = Arc::new(resource);
        if self
            .resources
            .iter()
            .any(|found| found.name() == resource.name())
        {
            tracing::warn!(
                target: "soot",
                "resource `{}` is already registered, replacing it",
                resource.name()
            );
            self.resources
                .retain(|found| found.name() != resource.name());
        }
        self.resources.push(resource);
        self
    }

    /// Register a resource type, which supplies its own declaration.
    pub fn add_resource<T: crate::resource_impl::Resource>(&mut self) -> &mut Self {
        self.push(T::definition())
    }

    pub fn resources(&self) -> &[Arc<ResourceDef>] {
        &self.resources
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Look up a resource by name.
    pub fn resource(&self, name: &str) -> Result<&Arc<ResourceDef>> {
        self.resources
            .iter()
            .find(|resource| resource.name() == name)
            .ok_or_else(|| {
                Error::changeset(format!(
                    "no resource named `{name}` is registered in the domain"
                ))
                .into()
            })
    }

    pub fn has_resource(&self, name: &str) -> bool {
        self.resources
            .iter()
            .any(|resource| resource.name() == name)
    }

    /// Every registered resource's name and table, for a quick overview.
    pub fn tables(&self) -> BTreeMap<String, String> {
        self.resources
            .iter()
            .map(|resource| {
                (
                    resource.name().to_string(),
                    resource.table_name().to_string(),
                )
            })
            .collect()
    }

    /// Run a create action.
    ///
    /// `action` may be omitted to use the resource's primary create action.
    pub async fn create(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        resource: &str,
        action: Option<&str>,
        params: BTreeMap<String, FieldValue>,
        context: &dyn ActionContext,
    ) -> Result<Record> {
        let definition = self.resource(resource)?.clone();
        let action = definition.resolve_action(action, ActionType::Create)?;
        let arguments = action.resolve_arguments(&no_arguments())?;
        let changeset = Changeset::for_create(definition, action, params, arguments)?;
        run_changeset_with(data_layer, changeset, context).await
    }

    /// Run an update action against the record identified by `id`.
    pub async fn update(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        resource: &str,
        action: Option<&str>,
        id: &str,
        params: BTreeMap<String, FieldValue>,
        context: &dyn ActionContext,
    ) -> Result<Record> {
        let definition = self.resource(resource)?.clone();
        let action = definition.resolve_action(action, ActionType::Update)?;
        let arguments = action.resolve_arguments(&BTreeMap::new())?;

        let record = self
            .find_record(data_layer, &definition, &action, id)
            .await?
            .ok_or_else(|| {
                Errors::from(Error::changeset(format!(
                    "no `{resource}` record with id {id}"
                )))
            })?;

        let changeset = Changeset::for_update(definition, action, record, params, arguments)?;
        run_changeset_with(data_layer, changeset, context).await
    }

    /// Run a destroy action against the record identified by `id`.
    pub async fn destroy(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        resource: &str,
        action: Option<&str>,
        id: &str,
        context: &dyn ActionContext,
    ) -> Result<Record> {
        let definition = self.resource(resource)?.clone();
        let action = definition.resolve_action(action, ActionType::Destroy)?;
        let arguments = action.resolve_arguments(&BTreeMap::new())?;

        let record = self
            .find_record(data_layer, &definition, &action, id)
            .await?
            .ok_or_else(|| {
                Errors::from(Error::changeset(format!(
                    "no `{resource}` record with id {id}"
                )))
            })?;

        let changeset = Changeset::for_destroy(definition, action, record, arguments)?;
        run_changeset_with(data_layer, changeset, context).await
    }

    /// Run a read action, returning every matching record.
    pub async fn read(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        resource: &str,
        action: Option<&str>,
        configure: impl FnOnce(&mut Query),
        context: &dyn ActionContext,
    ) -> Result<Vec<Record>> {
        let query = self
            .build_query(resource, action, configure, context)
            .await?;
        data_layer.read(Arc::clone(query.resource()), &query).await
    }

    /// Run a read action restricted to one record.
    pub async fn read_one(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        resource: &str,
        action: Option<&str>,
        configure: impl FnOnce(&mut Query),
        context: &dyn ActionContext,
    ) -> Result<Option<Record>> {
        let query = self
            .build_query(resource, action, configure, context)
            .await?;
        Ok(data_layer
            .read(Arc::clone(query.resource()), &query)
            .await?
            .into_iter()
            .next())
    }

    /// Load more relationships onto records that are already in hand, the way
    /// Ash's `Ash.load` lets a caller follow up on an earlier read.
    pub async fn load(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        resource: &str,
        records: &mut [Record],
        relationships: &[&str],
    ) -> Result<()> {
        let definition = self.resource(resource)?.clone();
        let names: Vec<String> = relationships.iter().map(|name| name.to_string()).collect();
        data_layer.load(definition, records, &names).await
    }

    /// Run a generic action that is not scoped to a record.
    ///
    /// The action's implementation sees the resolved arguments, the ambient
    /// context, and a handle on the data layer, but no record — there is no
    /// record to scope it to. Use [`Domain::run_generic_on`] when there is one.
    pub async fn run_generic(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        resource: &str,
        action: &str,
        arguments: BTreeMap<String, FieldValue>,
        context: &dyn ActionContext,
    ) -> Result<FieldValue> {
        self.run_generic_inner(data_layer, resource, action, arguments, context, None)
            .await
    }

    /// Run a generic action against one record, by primary key.
    ///
    /// This is Ash's generic action on a resource that takes a record: the
    /// implementation gets [`GenericInput::data`], so it can read the current
    /// state, and it has the data layer, so it can read and write whatever else
    /// it needs.
    pub async fn run_generic_on(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        resource: &str,
        action: &str,
        id: &str,
        arguments: BTreeMap<String, FieldValue>,
        context: &dyn ActionContext,
    ) -> Result<FieldValue> {
        let definition = self.resource(resource)?.clone();
        let lookup = definition.find_action(action).cloned().ok_or_else(|| {
            Errors::from(Error::changeset(format!(
                "no action named `{action}` on `{}`",
                definition.name()
            )))
        })?;
        let record = self
            .find_record(data_layer, &definition, &lookup, id)
            .await?
            .ok_or_else(|| {
                Errors::from(Error::changeset(format!(
                    "no `{resource}` record with id {id}"
                )))
            })?;
        self.run_generic_inner(
            data_layer,
            resource,
            action,
            arguments,
            context,
            Some(record),
        )
        .await
    }

    /// Shared body of the two generic entry points.
    async fn run_generic_inner(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        resource: &str,
        action: &str,
        arguments: BTreeMap<String, FieldValue>,
        context: &dyn ActionContext,
        data: Option<Record>,
    ) -> Result<FieldValue> {
        let definition = self.resource(resource)?.clone();
        let action = definition
            .find_action(action)
            .ok_or_else(|| {
                Errors::from(Error::changeset(format!(
                    "no action named `{action}` on `{}`",
                    definition.name()
                )))
            })?
            .clone();

        if action.action_type() != ActionType::Generic {
            return Err(Error::changeset(format!(
                "`{}` is a {} action, not a generic one",
                action.name(),
                action.action_type().as_str()
            ))
            .into());
        }

        let runner = action.runner().cloned().ok_or_else(|| {
            Errors::from(Error::changeset(format!(
                "generic action `{}` has no implementation",
                action.name()
            )))
        })?;

        let arguments = action.resolve_arguments(&arguments).map_err(Errors::from)?;
        let mut input = crate::generic::GenericInput::new(
            Arc::clone(&definition),
            Arc::new(action),
            arguments,
            context.handle(),
            Arc::clone(data_layer),
        );
        if let Some(record) = data {
            input = input.with_data(record);
        }
        runner(input).await
    }

    /// Build a query for a read action, running its preparations first.
    pub async fn build_query(
        &self,
        resource: &str,
        action: Option<&str>,
        configure: impl FnOnce(&mut Query),
        context: &dyn ActionContext,
    ) -> Result<Query> {
        let definition = self.resource(resource)?.clone();
        let action = definition.resolve_action(action, ActionType::Read)?;

        if action.action_type() != ActionType::Read {
            return Err(Error::changeset(format!(
                "`{}` is a {} action, not a read action",
                action.name(),
                action.action_type().as_str()
            ))
            .into());
        }

        let mut query = Query::new(Arc::clone(&definition), Arc::clone(&action));
        configure(&mut query);
        run_preparations(&mut query, &action, context).await?;
        query.validate()?;
        Ok(query)
    }

    /// Read a single record by primary key, ignoring the resource's read
    /// action entirely. Used to fetch the current state before a write.
    async fn find_record(
        &self,
        data_layer: &Arc<dyn DataLayer>,
        definition: &Arc<ResourceDef>,
        action: &Action,
        id: &str,
    ) -> Result<Option<Record>> {
        let mut query = Query::new(Arc::clone(definition), Arc::new(action.clone()));
        query.flag(INCLUDE_DELETED_FLAG);
        query.filter_eq(definition.primary_key_column(), coerce_id(definition, id));
        Ok(data_layer
            .read(Arc::clone(definition), &query)
            .await?
            .into_iter()
            .next())
    }

    /// A summary of the whole domain: resources, their attributes, actions and
    /// interfaces. This is the introspection Ash relies on to generate an API
    /// without the developer writing it.
    pub fn describe(&self) -> DomainDescription {
        DomainDescription {
            resources: self
                .resources
                .iter()
                .map(|resource| ResourceDescription {
                    name: resource.name().to_string(),
                    table: resource.table_name().to_string(),
                    description: resource.description().map(|d| d.to_string()),
                    primary_key: resource.primary_key_column().to_string(),
                    timestamps: resource.has_timestamps(),
                    soft_deletable: resource.is_soft_deletable(),
                    attributes: resource
                        .attributes()
                        .iter()
                        .map(|attribute| AttributeDescription {
                            name: attribute.name().to_string(),
                            column: attribute.column_name().to_string(),
                            ty: attribute.ty().field_type_label(),
                            primary_key: attribute.is_primary_key(),
                            required: !attribute.allows_nil(),
                            sensitive: attribute.is_sensitive(),
                            public: attribute.is_public(),
                            writable: attribute.is_writable(),
                        })
                        .collect(),
                    relationships: resource
                        .relationships()
                        .iter()
                        .map(|relationship| RelationshipDescription {
                            name: relationship.name().to_string(),
                            ty: relationship.relationship_type().as_str().to_string(),
                            destination: relationship.destination().to_string(),
                            source_attribute: relationship.source_attribute().to_string(),
                            destination_attribute: relationship.destination_attribute().to_string(),
                            writable: relationship.is_writable(),
                        })
                        .collect(),
                    calculations: resource
                        .calculations()
                        .iter()
                        .map(|calculation| calculation.name().to_string())
                        .collect(),
                    aggregates: resource
                        .aggregates()
                        .iter()
                        .map(|aggregate| aggregate.name().to_string())
                        .collect(),
                    actions: resource
                        .actions()
                        .iter()
                        .map(|action| ActionDescription {
                            name: action.name().to_string(),
                            ty: action.action_type().as_str().to_string(),
                            primary: action.is_primary(),
                            in_transaction: action.runs_in_transaction(),
                            accept: action.accepted().to_vec(),
                            arguments: action
                                .arguments()
                                .iter()
                                .map(|argument| ArgumentDescription {
                                    name: argument.name().to_string(),
                                    ty: argument.ty().field_type_label(),
                                    required: argument.is_required(),
                                    private: argument.is_private(),
                                })
                                .collect(),
                            changes: action
                                .changes()
                                .iter()
                                .map(|entity| {
                                    entity
                                        .get()
                                        .describe()
                                        .unwrap_or_else(|| "change".to_string())
                                })
                                .collect(),
                            validations: action
                                .validations()
                                .iter()
                                .map(|entity| {
                                    entity
                                        .get()
                                        .describe()
                                        .unwrap_or_else(|| "validation".to_string())
                                })
                                .collect(),
                            preparations: action
                                .preparations()
                                .iter()
                                .map(|entity| {
                                    entity
                                        .get()
                                        .describe()
                                        .unwrap_or_else(|| "preparation".to_string())
                                })
                                .collect(),
                            description: action.description().map(|d| d.to_string()),
                        })
                        .collect(),
                    interfaces: resource
                        .interfaces()
                        .iter()
                        .map(|interface| InterfaceDescription {
                            name: interface.name().to_string(),
                            entries: interface
                                .entries()
                                .iter()
                                .map(|entry| InterfaceEntryDescription {
                                    name: entry.name().to_string(),
                                    action: entry.action().to_string(),
                                    arguments: entry.arguments().to_vec(),
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

/// The empty argument map, for an action that declares no arguments.
fn no_arguments() -> BTreeMap<String, FieldValue> {
    BTreeMap::new()
}

/// Coerce a primary key supplied as a string into the resource's declared type,
/// so `domain.update(..., "3", ...)` works against a UUID keyed resource too.
fn coerce_id(definition: &ResourceDef, id: &str) -> FieldValue {
    definition
        .primary_key_type()
        .coerce(FieldValue::String(id.to_string()))
}

/// Run a prepared query's preparations, then validate it.
///
/// Preparations run in declaration order and each one sees the query as the
/// previous left it, which is what lets a preparation build on a filter an
/// earlier one added.
pub async fn run_preparations(
    query: &mut Query,
    action: &Action,
    context: &dyn ActionContext,
) -> Result<()> {
    let mut errors = ErrorList::new();
    for entity in action.preparations() {
        if !entity.condition().holds_for_query(query) {
            continue;
        }
        if let Err(found) = entity.get().prepare(query, context).await {
            errors.add_errors(found);
        }
    }
    errors.into_result()
}

/// Run an action's changes, then its validations, then the data layer write.
///
/// This is the action lifecycle, and the order matters. Changes transform the
/// changeset, so validations must see the settled result; and constraint checks
/// come last so they see everything both stages produced.
pub async fn run_changeset(
    data_layer: &Arc<dyn DataLayer>,
    changeset: Changeset,
) -> Result<Record> {
    run_changeset_with(data_layer, changeset, &DefaultActionContext::new()).await
}

pub async fn run_changeset_with(
    data_layer: &Arc<dyn DataLayer>,
    changeset: Changeset,
    context: &dyn ActionContext,
) -> Result<Record> {
    let changeset = apply_changes(changeset, context).await?;
    apply_validations(&changeset, context).await?;
    changeset.check_constraints()?;

    if changeset.has_errors() {
        return Err(Errors::from(changeset.errors().clone()));
    }

    let resource = Arc::clone(changeset.resource());
    let loads: Vec<String> = changeset.loads().iter().cloned().collect();
    let selects: Vec<String> = changeset.selects().iter().cloned().collect();

    let mut record = match changeset.kind() {
        ChangesetKind::Create => data_layer.create(Arc::clone(&resource), &changeset).await?,
        ChangesetKind::Update => data_layer.update(Arc::clone(&resource), &changeset).await?,
        ChangesetKind::Destroy => {
            data_layer
                .destroy(Arc::clone(&resource), &changeset)
                .await?
        }
    };

    apply_calculations(&resource, &mut record).await;
    apply_aggregates(data_layer, &resource, &mut record).await?;

    if !loads.is_empty() {
        data_layer
            .load(
                Arc::clone(&resource),
                std::slice::from_mut(&mut record),
                &loads,
            )
            .await?;
    }

    if !selects.is_empty() {
        let mut keep = std::collections::BTreeSet::new();
        keep.insert(resource.primary_key_column().to_string());
        for attribute in &selects {
            keep.insert(
                resource
                    .find_attribute(attribute)
                    .map(|found| found.column_name().to_string())
                    .unwrap_or_else(|| attribute.clone()),
            );
        }
        record.select_columns(&keep);
    }

    Ok(record)
}

/// Run every applicable change in declaration order.
pub async fn apply_changes(
    mut changeset: Changeset,
    context: &dyn ActionContext,
) -> Result<Changeset> {
    let action = Arc::clone(changeset.action());
    for entity in action.changes() {
        if !entity.condition().holds_for_changeset(&changeset) {
            continue;
        }
        if !entity.get().has_change(&changeset) {
            continue;
        }
        match entity.get().change(changeset, context).await {
            Ok(next) => changeset = next,
            Err(errors) => {
                // A change that fails outright stops the pipeline. Its errors
                // are reported as they came, since the changeset it would have
                // returned does not exist.
                return Err(errors);
            }
        }
    }
    Ok(changeset)
}

/// Run every applicable validation in declaration order, collecting all errors
/// rather than stopping at the first.
pub async fn apply_validations(changeset: &Changeset, context: &dyn ActionContext) -> Result<()> {
    let mut errors = ErrorList::new();
    for entity in changeset.action().validations() {
        if !entity.condition().holds_for_changeset(changeset) {
            continue;
        }
        if let Err(found) = entity.get().validate(changeset, context).await {
            errors.add_errors(found);
        }
    }
    errors.into_result()
}

/// Run the resource's calculations against a record.
pub async fn apply_calculations(resource: &ResourceDef, record: &mut Record) {
    for calculation in resource.calculations() {
        let name = calculation.name().to_string();
        // A calculation that cannot complete is left off the record rather than
        // failing the whole action, since it is derived data.
        match calculation.calculate(record.clone()).await {
            Ok(value) => record.put_calculated(&name, value),
            Err(errors) => {
                tracing::debug!(
                    target: "soot",
                    "calculation `{name}` on `{}` failed: {errors}",
                    resource.name()
                );
            }
        }
    }
}

/// Compute the named calculations on a record.
///
/// A calculation the resource does not declare, or that is not public, is
/// skipped: a query may name one from a trusted caller, but a record should not
/// expose a value the resource declared private.
pub async fn apply_selected_calculations(
    resource: &ResourceDef,
    names: &BTreeSet<String>,
    record: &mut Record,
) {
    for calculation in resource.calculations() {
        if !calculation.is_public() {
            continue;
        }
        if !names.contains(calculation.name()) {
            continue;
        }
        let name = calculation.name().to_string();
        match calculation.calculate(record.clone()).await {
            Ok(value) => record.put_calculated(&name, value),
            Err(errors) => {
                tracing::debug!(
                    target: "soot",
                    "calculation `{name}` on `{}` failed: {errors}",
                    resource.name()
                );
            }
        }
    }
}

/// Run the resource's aggregates against a record.
pub async fn apply_aggregates(
    data_layer: &Arc<dyn DataLayer>,
    resource: &Arc<ResourceDef>,
    record: &mut Record,
) -> Result<()> {
    for aggregate in resource.aggregates() {
        let query = Query::new(
            Arc::clone(resource),
            Arc::new(crate::action::Action::read("read")),
        );
        if let Some(value) = data_layer
            .aggregate(Arc::clone(resource), &query, aggregate)
            .await?
        {
            record.put_aggregated(aggregate.name(), value);
        }
    }
    Ok(())
}

// ---- introspection types ---------------------------------------------------

#[derive(Debug, Clone)]
pub struct DomainDescription {
    pub resources: Vec<ResourceDescription>,
}

#[derive(Debug, Clone)]
pub struct ResourceDescription {
    pub name: String,
    pub table: String,
    pub description: Option<String>,
    pub primary_key: String,
    pub timestamps: bool,
    pub soft_deletable: bool,
    pub attributes: Vec<AttributeDescription>,
    pub relationships: Vec<RelationshipDescription>,
    pub calculations: Vec<String>,
    pub aggregates: Vec<String>,
    pub actions: Vec<ActionDescription>,
    pub interfaces: Vec<InterfaceDescription>,
}

#[derive(Debug, Clone)]
pub struct AttributeDescription {
    pub name: String,
    pub column: String,
    pub ty: String,
    pub primary_key: bool,
    pub required: bool,
    pub sensitive: bool,
    pub public: bool,
    pub writable: bool,
}

#[derive(Debug, Clone)]
pub struct RelationshipDescription {
    pub name: String,
    pub ty: String,
    pub destination: String,
    pub source_attribute: String,
    pub destination_attribute: String,
    pub writable: bool,
}

#[derive(Debug, Clone)]
pub struct ActionDescription {
    pub name: String,
    pub ty: String,
    pub primary: bool,
    pub in_transaction: bool,
    pub accept: Vec<String>,
    pub arguments: Vec<ArgumentDescription>,
    pub changes: Vec<String>,
    pub validations: Vec<String>,
    pub preparations: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArgumentDescription {
    pub name: String,
    pub ty: String,
    pub required: bool,
    pub private: bool,
}

#[derive(Debug, Clone)]
pub struct InterfaceDescription {
    pub name: String,
    pub entries: Vec<InterfaceEntryDescription>,
}

#[derive(Debug, Clone)]
pub struct InterfaceEntryDescription {
    pub name: String,
    pub action: String,
    pub arguments: Vec<String>,
}
