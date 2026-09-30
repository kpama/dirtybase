use std::{collections::BTreeMap, sync::Arc};

use dirtybase_db::{
    base::{
        column::ColumnType,
        table::{CREATED_AT_FIELD, DELETED_AT_FIELD, UPDATED_AT_FIELD},
    },
    field_values::FieldValue,
};

use crate::{
    action::{Action, ActionType},
    aggregate::Aggregate,
    attribute::{Attribute, AttributeType},
    error::{Error, Errors, Result},
    extension::ExtensionRef,
    pipeline::Pipeline,
    relationship::{Calculation, Relationship, RelationshipType},
};

/// The declaration of a resource: everything there is to know about a domain
/// entity, and nothing about how it is stored.
///
/// A resource is data, not code. Once a `ResourceDef` exists, the generic engine
/// can run any of its actions, validate input, derive a table schema, and
/// describe the resource to a caller, without knowing the resource's type. That
/// is the property that makes the rest of the framework possible.
#[derive(Clone)]
pub struct ResourceDef {
    name: String,
    table: String,
    attributes: Vec<Attribute>,
    relationships: Vec<Relationship>,
    calculations: Vec<Calculation>,
    aggregates: Vec<Aggregate>,
    actions: Vec<Action>,
    pipelines: Vec<Pipeline>,
    interfaces: Vec<InterfaceDefinition>,
    extensions: Vec<ExtensionRef>,
    metadata: BTreeMap<String, FieldValue>,
    timestamps: bool,
    soft_deletable: bool,
    description: Option<String>,
}

impl std::fmt::Debug for ResourceDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceDef")
            .field("name", &self.name)
            .field("table", &self.table)
            .field("attributes", &self.attributes)
            .field("relationships", &self.relationships)
            .field("actions", &self.actions)
            .finish_non_exhaustive()
    }
}

impl ResourceDef {
    /// Start a resource named `name`, with `name` as the table name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            table: name.to_string(),
            attributes: Vec::new(),
            relationships: Vec::new(),
            calculations: Vec::new(),
            aggregates: Vec::new(),
            actions: Vec::new(),
            pipelines: Vec::new(),
            interfaces: Vec::new(),
            extensions: Vec::new(),
            metadata: BTreeMap::new(),
            timestamps: false,
            soft_deletable: false,
            description: None,
        }
    }

    /// Store this resource in a differently named table.
    pub fn table(mut self, table: &str) -> Self {
        self.table = table.to_string();
        self
    }

    /// Add a `created_at` and `updated_at` column, maintained on every write.
    pub fn timestamps(mut self) -> Self {
        self.timestamps = true;
        self
    }

    /// Add a `deleted_at` column. Destroy actions then soft delete by default.
    pub fn soft_deletable(mut self) -> Self {
        self.soft_deletable = true;
        self
    }

    /// An id attribute of the given type, marked as the primary key.
    pub fn primary_key(self, ty: AttributeType) -> Self {
        self.attribute(Attribute::new("id", ty).primary_key())
    }

    /// A UUID v7 primary key, which is what most dirtybase resources use.
    pub fn uuid_primary_key(self) -> Self {
        self.primary_key(AttributeType::Uuid)
    }

    /// An auto-incrementing integer primary key.
    pub fn integer_primary_key(self) -> Self {
        self.primary_key(AttributeType::Integer)
    }

    pub fn attribute(mut self, attribute: Attribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Append an attribute in place.
    ///
    /// [`ResourceDef::attribute`] consumes the resource, so an extension
    /// extending a declaration it was handed by reference needs this instead.
    /// A duplicate name is appended rather than rejected, matching
    /// [`ResourceDef::attribute`]; the first declaration is the one
    /// [`ResourceDef::find_attribute`] returns.
    pub fn add_attribute(&mut self, attribute: Attribute) -> &mut Self {
        self.attributes.push(attribute);
        self
    }

    pub fn with_attributes(mut self, attributes: impl IntoIterator<Item = Attribute>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    /// Declare a relationship.
    ///
    /// A `belongs_to` holds its foreign key on *this* table, so declaring one
    /// implies a column here. Rather than making the caller declare the same
    /// name twice, the key is added as an attribute automatically — typed
    /// `uuid`, which is what the vast majority of primary keys are. Declaring an
    /// attribute of that name yourself, before or after, wins, so a non-uuid key
    /// is still expressible:
    ///
    /// ```
    /// # use dirtybase_soot::prelude::*;
    /// // A uuid foreign key, implied.
    /// let post = ResourceDef::new("Post")
    ///     .uuid_primary_key()
    ///     .relationship(Relationship::belongs_to("author", "User", "author_id"));
    /// assert!(post.find_attribute("author_id").is_some());
    ///
    /// // An integer foreign key, declared explicitly.
    /// let ticket = ResourceDef::new("Ticket")
    ///     .integer_primary_key()
    ///     .attribute(Attribute::integer("user_id").required())
    ///     .relationship(Relationship::belongs_to("user", "User", "user_id"));
    /// assert_eq!(
    ///     ticket.find_attribute("user_id").map(|a| a.ty().clone()),
    ///     Some(AttributeType::Integer),
    /// );
    /// ```
    pub fn relationship(mut self, relationship: Relationship) -> Self {
        // Only a `belongs_to` puts a key on this side. A `has_one`/`has_many`
        // keys off this table's primary key, which already exists.
        if relationship.relationship_type() == RelationshipType::BelongsTo {
            let key = relationship.source_attribute().to_string();
            if !self.attributes.iter().any(|a| a.name() == key) {
                self.attributes
                    .push(Attribute::uuid(&key).describe("Implied by a belongs_to relationship"));
            }
        }
        self.relationships.push(relationship);
        self
    }

    pub fn calculation(mut self, calculation: Calculation) -> Self {
        self.calculations.push(calculation);
        self
    }

    pub fn aggregate(mut self, aggregate: Aggregate) -> Self {
        self.aggregates.push(aggregate);
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_actions(mut self, actions: impl IntoIterator<Item = Action>) -> Self {
        self.actions.extend(actions);
        self
    }

    /// Add a reusable pipeline that actions on this resource can pipe through.
    pub fn pipeline(mut self, pipeline: Pipeline) -> Self {
        self.pipelines.push(pipeline);
        self
    }

    /// Attach the named pipeline to every action of the given types that does
    /// not already include it. This is the practical way to get resource wide
    /// behaviour such as stamping `updated_at` without repeating it per action.
    pub fn global_pipeline(mut self, pipeline: Pipeline, on: &[ActionType]) -> Self {
        let name = pipeline.name().to_string();
        for action in self.actions.iter_mut() {
            if on.contains(&action.action_type()) && !action.mentions_pipeline(&name) {
                action.apply_pipeline(&pipeline);
            }
        }
        self.pipelines.push(pipeline);
        self
    }

    /// The default action set: a primary read, create, update and destroy, with
    /// create and update accepting every writable attribute.
    ///
    /// Equivalent to Ash's `defaults [:read, create: :*, update: :*]`.
    pub fn default_actions(self) -> Self {
        self.action(Action::read("read").primary().describe("Read any record"))
            .action(
                Action::create("create")
                    .primary()
                    .accept_all_writable()
                    .describe("Create a record"),
            )
            .action(
                Action::update("update")
                    .primary()
                    .accept_all_writable()
                    .describe("Update a record"),
            )
            .action(
                Action::destroy("destroy")
                    .primary()
                    .describe("Destroy a record"),
            )
    }

    /// Publish a code interface so callers get a typed function per action
    /// rather than a string-keyed call.
    pub fn define_interface(mut self, interface: InterfaceDefinition) -> Self {
        self.interfaces.push(interface);
        self
    }

    /// Attach an extension to this resource.
    ///
    /// The extension's [`crate::extension::SootExtension::extend`] runs
    /// immediately, so anything it adds is part of the declaration from here on.
    /// Its runtime hooks then run around every action on this resource.
    ///
    /// ```
    /// # use dirtybase_soot::prelude::*;
    /// # use dirtybase_soot::extension::extension;
    /// let post = ResourceDef::new("Post")
    ///     .uuid_primary_key()
    ///     .extension(extension("stamp").describe("stamps `updated_by`"))
    ///     .default_actions();
    /// assert_eq!(post.extensions().len(), 1);
    /// ```
    pub fn extension<E: crate::extension::SootExtension + 'static>(mut self, extension: E) -> Self {
        self.add_extension(extension);
        self
    }

    /// Attach an extension in place. See [`ResourceDef::extension`].
    pub fn add_extension<E: crate::extension::SootExtension + 'static>(
        &mut self,
        extension: E,
    ) -> &mut Self {
        extension.extend(self);
        self.extensions.push(Arc::new(extension));
        self
    }

    /// Attach several extensions.
    pub fn with_extensions(mut self, extensions: impl IntoIterator<Item = ExtensionRef>) -> Self {
        for extension in extensions {
            extension.extend(&mut self);
            self.extensions.push(extension);
        }
        self
    }

    pub fn extensions(&self) -> &[ExtensionRef] {
        &self.extensions
    }

    pub fn find_extension(&self, name: &str) -> Option<&ExtensionRef> {
        self.extensions
            .iter()
            .find(|extension| extension.name() == name)
    }

    /// Whether an extension of that name is attached, which is the check an
    /// extension itself makes before contributing twice.
    pub fn has_extension(&self, name: &str) -> bool {
        self.find_extension(name).is_some()
    }

    pub fn with_metadata(mut self, key: &str, value: impl Into<FieldValue>) -> Self {
        self.metadata.insert(key.to_string(), value.into());
        self
    }

    pub fn describe(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    // ---- lookup ------------------------------------------------------------
    //
    // Singular lookups are named `find_*` because the singular names are taken
    // by the builders: `resource.attribute(a)` adds an attribute, and
    // `resource.find_attribute("a")` gets it back. The plural getters have no
    // such conflict.

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn table_name(&self) -> &str {
        &self.table
    }

    pub fn find_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.name() == name)
    }

    /// Whether the resource declares that attribute.
    ///
    /// An extension that contributes a column needs this before adding it, since
    /// a resource that already declares the name — to type it differently, or
    /// to make it required — must not have a second declaration appended.
    pub fn has_attribute(&self, name: &str) -> bool {
        self.find_attribute(name).is_some()
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    pub fn find_relationship(&self, name: &str) -> Option<&Relationship> {
        self.relationships.iter().find(|r| r.name() == name)
    }

    pub fn relationships(&self) -> &[Relationship] {
        &self.relationships
    }

    pub fn find_calculation(&self, name: &str) -> Option<&Calculation> {
        self.calculations.iter().find(|c| c.name() == name)
    }

    pub fn calculations(&self) -> &[Calculation] {
        &self.calculations
    }

    pub fn find_aggregate(&self, name: &str) -> Option<&Aggregate> {
        self.aggregates.iter().find(|a| a.name() == name)
    }

    pub fn aggregates(&self) -> &[Aggregate] {
        &self.aggregates
    }

    pub fn pipelines(&self) -> &[Pipeline] {
        &self.pipelines
    }

    pub fn find_pipeline(&self, name: &str) -> Option<&Pipeline> {
        self.pipelines.iter().find(|p| p.name() == name)
    }

    pub fn interfaces(&self) -> &[InterfaceDefinition] {
        &self.interfaces
    }

    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// The declared actions, mutably.
    ///
    /// An extension that has to change the actions already declared — adding a
    /// change to every create, say — needs this rather than a way to append,
    /// because the actions it wants are the ones the resource built earlier.
    pub fn actions_mut(&mut self) -> &mut [Action] {
        &mut self.actions
    }

    /// Append an action in place, for an extension extending the declaration.
    pub fn add_action(&mut self, action: Action) -> &mut Self {
        self.actions.push(action);
        self
    }

    pub fn find_action(&self, name: &str) -> Option<&Action> {
        self.actions.iter().find(|a| a.name() == name)
    }

    pub fn metadata(&self) -> &BTreeMap<String, FieldValue> {
        &self.metadata
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn has_timestamps(&self) -> bool {
        self.timestamps
    }

    pub fn is_soft_deletable(&self) -> bool {
        self.soft_deletable
    }

    pub fn created_at_column(&self) -> &str {
        CREATED_AT_FIELD
    }

    pub fn updated_at_column(&self) -> &str {
        UPDATED_AT_FIELD
    }

    pub fn deleted_at_column(&self) -> &str {
        DELETED_AT_FIELD
    }

    /// The destination table and column a `belongs_to` attribute points at, if
    /// this attribute is the key of one.
    ///
    /// The schema builder needs this to declare a real foreign key, and it can
    /// only be answered by the resource itself, since only the resource knows
    /// which of its attributes a relationship is anchored on.
    pub fn relationship_target_column(&self, attribute_name: &str) -> Option<(&str, &str)> {
        let relationship = self.relationships.iter().find(|relationship| {
            relationship.relationship_type() == RelationshipType::BelongsTo
                && relationship.source_attribute() == attribute_name
        })?;

        // The destination's own column is only known if the destination resource
        // happens to use the conventional name, which the caller resolves
        // against the registered destination.
        Some((
            relationship.destination(),
            relationship.destination_attribute(),
        ))
    }

    /// The primary key attribute, which the resource requires.
    pub fn primary_key_attribute(&self) -> std::result::Result<&Attribute, Error> {
        self.attributes
            .iter()
            .find(|a| a.is_primary_key())
            .ok_or_else(|| {
                Error::no_primary_key(format!(
                    "resource `{}` has no primary key attribute",
                    self.name
                ))
            })
    }

    /// The primary key's column name.
    pub fn primary_key_column(&self) -> &str {
        self.attributes
            .iter()
            .find(|a| a.is_primary_key())
            .map(|a| a.column_name())
            .unwrap_or("id")
    }

    /// The primary key attribute's type.
    pub fn primary_key_type(&self) -> AttributeType {
        self.attributes
            .iter()
            .find(|a| a.is_primary_key())
            .map(|a| a.ty().clone())
            .unwrap_or(AttributeType::Uuid)
    }

    /// The action to use when the caller does not name one.
    pub fn primary_action(&self, ty: ActionType) -> Option<&Action> {
        self.actions
            .iter()
            .find(|a| a.is_primary() && a.action_type() == ty)
    }

    /// Resolve an action by name, falling back to the primary action of the
    /// requested type when the name is empty or `None`.
    pub fn resolve_action(&self, name: Option<&str>, ty: ActionType) -> Result<Arc<Action>> {
        match name {
            Some(name) if !name.is_empty() => self
                .find_action(name)
                .map(|action| Arc::new(action.clone()))
                .ok_or_else(|| {
                    Errors::from(Error::changeset(format!(
                        "resource `{}` has no action named `{}`",
                        self.name, name
                    )))
                }),
            _ => self
                .primary_action(ty)
                .map(|action| Arc::new(action.clone()))
                .ok_or_else(|| {
                    Errors::from(Error::changeset(format!(
                        "resource `{}` has no primary {} action and none was named",
                        self.name,
                        ty.as_str()
                    )))
                }),
        }
    }

    /// The attributes an action would write, for introspection.
    pub fn accepted_attributes(&self, action: &Action) -> Vec<&str> {
        let mut out = Vec::new();
        for name in action.accepted() {
            if name == "*" {
                for attribute in &self.attributes {
                    if attribute.is_writable() {
                        out.push(attribute.name());
                    }
                }
                continue;
            }
            if let Some(attribute) = self.find_attribute(name) {
                if attribute.is_writable() {
                    out.push(attribute.name());
                }
                continue;
            }
            if self.find_relationship(name).is_some() {
                // Relationships are writable but carry a runtime value, so they
                // are reported separately rather than as an attribute name.
                continue;
            }
        }
        out
    }

    /// The column type for every column this resource declares, including the
    /// ones implied by `timestamps` and `soft_deletable`.
    pub fn column_types(&self) -> BTreeMap<String, ColumnType> {
        let mut out = BTreeMap::new();
        for attribute in &self.attributes {
            out.insert(
                attribute.column_name().to_string(),
                attribute.ty().to_column_type(),
            );
        }
        if self.timestamps {
            out.insert(
                CREATED_AT_FIELD.to_string(),
                AttributeType::Timestamp.to_column_type(),
            );
            out.insert(
                UPDATED_AT_FIELD.to_string(),
                AttributeType::Timestamp.to_column_type(),
            );
        }
        if self.soft_deletable {
            out.insert(
                DELETED_AT_FIELD.to_string(),
                AttributeType::Timestamp.to_column_type(),
            );
        }
        out
    }
}

/// A named entry point onto a resource's actions.
///
/// This is Ash's `code_interface`. Declaring one says "callers of this resource
/// should be able to say `Post.publish(id)` and have it mean `run the publish
/// action`", which is what lets an API layer generate a typed surface from
/// resource metadata alone.
#[derive(Debug, Clone)]
pub struct InterfaceDefinition {
    name: String,
    entries: Vec<InterfaceEntry>,
}

impl InterfaceDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: Vec::new(),
        }
    }

    /// Expose the action under the interface's own name, taking no arguments.
    pub fn action(self, action: &str) -> Self {
        self.with_action(action, action, &[])
    }

    /// Expose the action under a different name, with the argument names in the
    /// order they should be passed.
    pub fn with_action(self, name: &str, action: &str, arguments: &[&str]) -> Self {
        let mut next = self;
        next.entries.push(InterfaceEntry {
            name: name.to_string(),
            action: action.to_string(),
            arguments: arguments.iter().map(|a| a.to_string()).collect(),
        });
        next
    }

    /// Expose an action taking a single id, which is the common shape.
    pub fn with_id(self, name: &str, action: &str) -> Self {
        self.with_action(name, action, &[crate::resource::INTERFACE_ID_ARGUMENT])
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entries(&self) -> &[InterfaceEntry] {
        &self.entries
    }
}

/// One function on a code interface.
#[derive(Debug, Clone)]
pub struct InterfaceEntry {
    name: String,
    action: String,
    arguments: Vec<String>,
}

impl InterfaceEntry {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }
}

/// The conventional name of the id argument on a code interface.
pub const INTERFACE_ID_ARGUMENT: &str = "id";
