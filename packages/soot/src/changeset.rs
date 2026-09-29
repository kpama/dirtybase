use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use dirtybase_db::{field_values::FieldValue, types::ColumnAndValue};

use crate::{
    action::{Action, ActionType},
    attribute::is_nil,
    error::{Error, ErrorList, Result},
    record::Record,
    resource::ResourceDef,
};

/// What an action is being run against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangesetKind {
    Create,
    Update,
    Destroy,
}

impl ChangesetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Destroy => "destroy",
        }
    }
}

/// The unit of work passed through an action's changes and validations, and
/// finally to the data layer.
///
/// Two maps matter here. `attributes` holds the new values: the fields the
/// caller supplied plus anything earlier changes added or overwrote. `data`
/// holds the record as it currently exists in the database, which is what
/// update and destroy actions compare against.
///
/// Keeping them separate is what makes a partial update possible: a caller can
/// set `name` without having to supply the rest of the record.
#[derive(Clone)]
pub struct Changeset {
    resource: Arc<ResourceDef>,
    action: Arc<Action>,
    kind: ChangesetKind,
    attributes: BTreeMap<String, FieldValue>,
    data: Record,
    arguments: BTreeMap<String, FieldValue>,
    context: BTreeMap<String, FieldValue>,
    errors: ErrorList,
    /// Names to load after the action succeeds.
    load: BTreeSet<String>,
    /// Extra columns to select on the write-back read.
    select: BTreeSet<String>,
    /// The attributes the caller supplied, captured before any change runs.
    ///
    /// An accept-list check has to run against this rather than against
    /// `attributes`, because a change is allowed to set an attribute the caller
    /// never provided.
    provided: BTreeSet<String>,
    /// Columns a change asked to drop, so they are neither written nor read back.
    removed: BTreeSet<String>,
}

impl std::fmt::Debug for Changeset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Changeset")
            .field("resource", &self.resource.name())
            .field("action", &self.action.name())
            .field("kind", &self.kind)
            .field("attributes", &self.attributes)
            .field("errors", &self.errors)
            .finish_non_exhaustive()
    }
}

impl Changeset {
    /// Build a changeset for a create action.
    ///
    /// Values in `params` are filtered through the action's accept list, so a
    /// caller cannot write an attribute the action does not expose, and coerced
    /// to the declared attribute type. Anything left over is reported as an
    /// unknown field, which is what turns a typo into an error instead of a
    /// silently ignored value.
    pub fn for_create(
        resource: Arc<ResourceDef>,
        action: Arc<Action>,
        params: BTreeMap<String, FieldValue>,
        arguments: BTreeMap<String, FieldValue>,
    ) -> Result<Self> {
        let accepted = accepted_attributes(&resource, &action);
        let (attributes, mut errors) = coerce_params(&resource, params, &accepted);

        for relationship in resource.relationships() {
            if !relationship.is_writable() || !accepted.contains(relationship.name()) {
                continue;
            }
            if let Some(value) = attributes.get(relationship.name()) {
                if is_nil(value) && !relationship.allows_nil() {
                    errors.push(Error::required(relationship.name(), "is required"));
                }
            } else if !relationship.allows_nil() {
                errors.push(Error::required(relationship.name(), "is required"));
            }
        }

        let provided = attributes.keys().cloned().collect();
        Ok(Self {
            resource,
            action,
            kind: ChangesetKind::Create,
            attributes,
            data: Record::new(),
            arguments,
            context: BTreeMap::new(),
            errors: ErrorList::from_vec(errors),
            load: BTreeSet::new(),
            select: BTreeSet::new(),
            provided,
            removed: BTreeSet::new(),
        })
    }

    /// Build a changeset for an update action, starting from the stored record.
    pub fn for_update(
        resource: Arc<ResourceDef>,
        action: Arc<Action>,
        data: Record,
        params: BTreeMap<String, FieldValue>,
        arguments: BTreeMap<String, FieldValue>,
    ) -> Result<Self> {
        let accepted = accepted_attributes(&resource, &action);
        let (attributes, errors) = coerce_params(&resource, params, &accepted);
        let provided = attributes.keys().cloned().collect();

        Ok(Self {
            resource,
            action,
            kind: ChangesetKind::Update,
            attributes,
            data,
            arguments,
            context: BTreeMap::new(),
            errors: ErrorList::from_vec(errors),
            load: BTreeSet::new(),
            select: BTreeSet::new(),
            provided,
            removed: BTreeSet::new(),
        })
    }

    /// Build a changeset for a destroy action. The stored record is enough.
    pub fn for_destroy(
        resource: Arc<ResourceDef>,
        action: Arc<Action>,
        data: Record,
        arguments: BTreeMap<String, FieldValue>,
    ) -> Result<Self> {
        Ok(Self {
            resource,
            action,
            kind: ChangesetKind::Destroy,
            attributes: BTreeMap::new(),
            data,
            arguments,
            context: BTreeMap::new(),
            errors: ErrorList::new(),
            load: BTreeSet::new(),
            select: BTreeSet::new(),
            provided: BTreeSet::new(),
            removed: BTreeSet::new(),
        })
    }

    pub fn resource(&self) -> &Arc<ResourceDef> {
        &self.resource
    }

    pub fn action(&self) -> &Arc<Action> {
        &self.action
    }

    pub fn kind(&self) -> ChangesetKind {
        self.kind
    }

    pub fn action_type(&self) -> ActionType {
        self.action.action_type()
    }

    /// A new attribute value, whether supplied by the caller or set by a change.
    pub fn get(&self, name: &str) -> Option<FieldValue> {
        self.attributes.get(name).cloned()
    }

    /// The stored value, falling back to the new value for create actions where
    /// there is no stored record yet.
    pub fn get_data(&self, name: &str) -> Option<FieldValue> {
        match self.kind {
            ChangesetKind::Create => self.get(name).or_else(|| self.data.get(name)),
            _ => self.data.get(name),
        }
    }

    /// The effective value after the action: the new value when set, otherwise
    /// whatever the record already had.
    pub fn effective(&self, name: &str) -> Option<FieldValue> {
        self.get(name).or_else(|| self.data.get(name))
    }

    pub fn is_nil(&self, name: &str) -> bool {
        match self.effective(name) {
            Some(value) => is_nil(&value),
            None => true,
        }
    }

    /// Set an attribute value, coercing it to the declared type.
    ///
    /// Rejects attributes the resource does not declare, so a change cannot
    /// smuggle an undeclared column into a write.
    pub fn set(&mut self, name: &str, value: impl Into<FieldValue>) -> Result<()> {
        // The declared type is read first so the immutable borrow of the
        // resource ends before the changeset is mutated.
        let ty = self
            .resource
            .find_attribute(name)
            .map(|attribute| attribute.ty().clone());
        self.set_typed(name, ty.as_ref(), value)
    }

    /// Set an attribute value with an explicit type, for writing values that
    /// belong to the destination of a relationship rather than this resource.
    pub fn set_typed(
        &mut self,
        name: &str,
        ty: Option<&crate::attribute::AttributeType>,
        value: impl Into<FieldValue>,
    ) -> Result<()> {
        let value = match ty {
            Some(ty) => ty.coerce(value.into()),
            None => value.into(),
        };
        self.attributes.insert(name.to_string(), value);
        Ok(())
    }

    /// Remove a pending attribute value, revealing the stored one.
    pub fn unset(&mut self, name: &str) {
        self.attributes.remove(name);
    }

    /// Remove an attribute from the changeset entirely.
    ///
    /// Unlike [`Changeset::unset`], which lets the stored value show through,
    /// this drops the attribute from the data as well, so it is not written.
    pub fn remove(&mut self, name: &str) {
        self.attributes.remove(name);
        if let Some(column) = self
            .resource
            .find_attribute(name)
            .map(|attribute| attribute.column_name().to_string())
        {
            self.removed.insert(column);
        }
    }

    /// The value of an attribute as text, for validations that check shape
    /// rather than type.
    pub fn text(&self, name: &str) -> Option<String> {
        match self.effective(name)? {
            FieldValue::String(value) => Some(value),
            // Every FieldValue renders to something, so a numeric attribute can
            // still be length- and pattern-checked.
            other => Some(other.to_string()),
        }
    }

    /// The value of a numeric attribute.
    pub fn number(&self, name: &str) -> Option<f64> {
        match self.effective(name)? {
            FieldValue::I64(value) => Some(value as f64),
            FieldValue::I32(value) => Some(value as f64),
            FieldValue::I16(value) => Some(value as f64),
            FieldValue::I8(value) => Some(value as f64),
            FieldValue::U32(value) => Some(value as f64),
            FieldValue::U64(value) => Some(value as f64),
            FieldValue::F64(value) => Some(value),
            _ => None,
        }
    }

    /// The attributes a caller actually supplied, before any change added to
    /// them. This is what an accept-list check has to run against: a change that
    /// sets an attribute has not made the caller provide it.
    pub fn provided_attributes(&self) -> BTreeSet<String> {
        self.provided.clone()
    }

    /// The values a caller supplied, keyed by attribute name.
    pub fn provided_values(&self) -> BTreeMap<String, FieldValue> {
        self.provided
            .iter()
            .filter_map(|name| self.data.get(name).map(|value| (name.clone(), value)))
            .collect()
    }

    /// Only write the given attributes, discarding everything else pending.
    pub fn force_set_attributes(&mut self, attributes: BTreeMap<String, FieldValue>) -> Result<()> {
        for (name, value) in attributes {
            self.set(&name, value)?;
        }
        Ok(())
    }

    pub fn argument(&self, name: &str) -> Option<FieldValue> {
        self.arguments.get(name).cloned()
    }

    pub fn arguments(&self) -> &BTreeMap<String, FieldValue> {
        &self.arguments
    }

    pub fn set_argument(&mut self, name: &str, value: impl Into<FieldValue>) {
        self.arguments.insert(name.to_string(), value.into());
    }

    /// A free form value threaded through the action.
    pub fn get_context(&self, key: &str) -> Option<FieldValue> {
        self.context.get(key).cloned()
    }

    pub fn set_context(&mut self, key: &str, value: impl Into<FieldValue>) {
        self.context.insert(key.to_string(), value.into());
    }

    pub fn context(&self) -> &BTreeMap<String, FieldValue> {
        &self.context
    }

    pub fn attributes(&self) -> &BTreeMap<String, FieldValue> {
        &self.attributes
    }

    /// The names of attributes this action will actually write.
    pub fn changed_attributes(&self) -> Vec<String> {
        self.attributes.keys().cloned().collect()
    }

    pub fn data(&self) -> &Record {
        &self.data
    }

    pub fn set_data(&mut self, data: Record) {
        self.data = data;
    }

    /// Record the stored primary key so the data layer can target the row.
    pub fn set_primary_key(&mut self, value: impl Into<FieldValue>) {
        let column = self.resource.primary_key_column().to_string();
        self.data.set(&column, value);
    }

    /// Read the stored primary key.
    pub fn primary_key(&self) -> Option<FieldValue> {
        self.data.primary_key(self.resource.primary_key_column())
    }

    /// The column map to write.
    ///
    /// Attribute names are mapped to column names here, which is the single
    /// place the mapping between the resource's attribute vocabulary and the
    /// database's column vocabulary happens.
    pub fn to_column_and_value(&self) -> ColumnAndValue {
        let mut out = ColumnAndValue::new();
        for (name, value) in &self.attributes {
            let column = self
                .resource
                .find_attribute(name)
                .map(|a| a.column_name().to_string())
                .unwrap_or_else(|| name.clone());
            if self.removed.contains(&column) {
                continue;
            }
            out.insert(column, value.clone());
        }
        out
    }

    /// Add an error. Returns the changeset so errors can be added inline.
    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn add_errors(&mut self, errors: ErrorList) {
        self.errors.add_errors(errors);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &ErrorList {
        &self.errors
    }

    /// Request a relationship be loaded once the action succeeds.
    pub fn load(&mut self, relationship: &str) {
        self.load.insert(relationship.to_string());
    }

    pub fn loads(&self) -> &BTreeSet<String> {
        &self.load
    }

    /// Restrict the write-back read to these attributes, leaving others out.
    pub fn select(&mut self, attributes: &[&str]) {
        for attribute in attributes {
            self.select.insert(attribute.to_string());
        }
    }

    pub fn selects(&self) -> &BTreeSet<String> {
        &self.select
    }

    /// Check every writable attribute's constraints, then return any errors.
    ///
    /// This runs after all validations, as Ash does, so attribute types and
    /// constraints are enforced once the changes have settled.
    pub fn check_constraints(&self) -> Result<()> {
        let mut errors = ErrorList::new();
        for (name, value) in &self.attributes {
            let Some(attribute) = self.resource.find_attribute(name) else {
                continue;
            };
            if !attribute.is_writable() {
                continue;
            }
            if let Err(found) = attribute.constraint().check(name, value) {
                errors.add_errors(found);
            }
        }
        errors.into_result()
    }
}

/// The attribute names an action may write.
///
/// `accept ["*"]` means every writable attribute that is not a relationship,
/// which is Ash's `accept [:*]`. Relationships only appear when explicitly
/// listed and marked writable.
fn accepted_attributes(resource: &ResourceDef, action: &Action) -> BTreeSet<String> {
    let mut accepted = BTreeSet::new();

    for name in action.accepted() {
        if name == "*" {
            for attribute in resource.attributes() {
                if attribute.is_writable() {
                    accepted.insert(attribute.name().to_string());
                }
            }
            continue;
        }
        if let Some(attribute) = resource.find_attribute(name) {
            if attribute.is_writable() {
                accepted.insert(name.clone());
            }
            continue;
        }
        if resource.find_relationship(name).is_some() {
            accepted.insert(name.clone());
        }
    }

    // The primary key is always writable so callers can target a row by id
    // even when the action does not list it.
    accepted.insert(resource.primary_key_column().to_string());

    accepted
}

/// Split incoming params into accepted, type-coerced values plus errors for
/// anything the action does not accept.
fn coerce_params(
    resource: &ResourceDef,
    params: BTreeMap<String, FieldValue>,
    accepted: &BTreeSet<String>,
) -> (BTreeMap<String, FieldValue>, Vec<Error>) {
    let mut attributes = BTreeMap::new();
    let mut errors = Vec::new();

    for (name, value) in params {
        if !accepted.contains(&name) {
            errors.push(Error::unknown_field(
                &name,
                format!(
                    "`{}` is not accepted by the `{}` action",
                    name,
                    resource.name()
                ),
            ));
            continue;
        }
        if let Some(attribute) = resource.find_attribute(&name) {
            attributes.insert(name, attribute.ty().coerce(value));
        } else {
            // A writable relationship carries its value through as-is; the data
            // layer decides how to act on it.
            attributes.insert(name, value);
        }
    }

    (attributes, errors)
}
