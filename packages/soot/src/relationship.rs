use std::{future::Future, pin::Pin, sync::Arc};

use dirtybase_db::field_values::FieldValue;
use futures::future::BoxFuture;

use crate::{
    attribute::AttributeType,
    error::{Error, Result},
    record::Record,
};

/// How records on two resources relate to each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    /// The destination holds a foreign key pointing back at this resource.
    BelongsTo,
    /// Exactly one destination record points back at this resource.
    HasOne,
    /// Many destination records point back at this resource.
    HasMany,
    /// Many-to-many through a join resource that holds two foreign keys.
    ManyToMany,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BelongsTo => "belongs_to",
            Self::HasOne => "has_one",
            Self::HasMany => "has_many",
            Self::ManyToMany => "many_to_many",
        }
    }

    /// A to-one relationship is loaded by issuing a single query, a to-many by
    /// fetching a list.
    pub fn is_to_one(&self) -> bool {
        matches!(self, Self::BelongsTo | Self::HasOne)
    }

    /// For to-one relationships, the join is made from this resource's own
    /// column. For to-many, it is made from the destination's foreign key.
    pub fn joins_from_source(&self) -> bool {
        matches!(self, Self::BelongsTo)
    }
}

/// Replaces the data layer's default loading for a relationship.
///
/// The loader receives the single source record and returns its related value.
/// The data layer issues the calls, so a custom loader does not have to think
/// about batching; it only has to produce the right value for one record.
pub type RelationshipLoader =
    Arc<dyn Fn(Record) -> BoxFuture<'static, Result<LoadedRelationship>> + Send + Sync>;

/// The result of loading a relationship for a set of records.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadedRelationship {
    /// One record, or nothing. Used by `belongs_to` and `has_one`.
    ToOne(Option<Record>),
    /// Many records, always a (possibly empty) list. Used by `has_many`.
    ToMany(Vec<Record>),
}

impl LoadedRelationship {
    pub fn as_to_one(&self) -> Option<&Record> {
        match self {
            Self::ToOne(record) => record.as_ref(),
            _ => None,
        }
    }

    pub fn as_to_many(&self) -> &[Record] {
        match self {
            Self::ToMany(records) => records,
            _ => &[],
        }
    }

    /// The field type this relationship's loaded value takes, used to type the
    /// cast in [`Record::get`].
    pub fn attribute_type(&self) -> AttributeType {
        match self {
            Self::ToOne(_) => AttributeType::Json,
            Self::ToMany(_) => AttributeType::JsonValue,
        }
    }
}

/// A declared relationship between two resources.
///
/// `source_attribute` and `destination_attribute` are the two sides of the join.
/// For `belongs_to` the source is this resource's foreign key; for `has_many` it
/// is the destination's foreign key and this resource's primary key.
#[derive(Clone)]
pub struct Relationship {
    name: String,
    ty: RelationshipType,
    destination: String,
    source_attribute: String,
    destination_attribute: String,
    attribute_type: AttributeType,
    writable: bool,
    public: bool,
    allow_nil: bool,
    description: Option<String>,
    /// For `many_to_many`, the join resource holding the two foreign keys.
    join_resource: Option<String>,
    loader: Option<RelationshipLoader>,
}

impl std::fmt::Debug for Relationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Relationship")
            .field("name", &self.name)
            .field("type", &self.ty)
            .field("destination", &self.destination)
            .field("source_attribute", &self.source_attribute)
            .field("destination_attribute", &self.destination_attribute)
            .field("custom_loader", &self.loader.is_some())
            .finish()
    }
}

impl Relationship {
    pub fn new(
        name: &str,
        ty: RelationshipType,
        destination: &str,
        source_attribute: &str,
        destination_attribute: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            ty,
            destination: destination.to_string(),
            source_attribute: source_attribute.to_string(),
            destination_attribute: destination_attribute.to_string(),
            attribute_type: AttributeType::Json,
            writable: false,
            public: true,
            allow_nil: true,
            description: None,
            join_resource: None,
            loader: None,
        }
    }

    /// This resource holds a foreign key, so at most one destination record.
    pub fn belongs_to(name: &str, destination: &str, foreign_key: &str) -> Self {
        Self::new(
            name,
            RelationshipType::BelongsTo,
            destination,
            foreign_key,
            "id",
        )
    }

    /// The destination holds a foreign key pointing at this resource.
    pub fn has_one(name: &str, destination: &str, foreign_key: &str) -> Self {
        Self::new(
            name,
            RelationshipType::HasOne,
            destination,
            "id",
            foreign_key,
        )
    }

    pub fn has_many(name: &str, destination: &str, foreign_key: &str) -> Self {
        Self::new(
            name,
            RelationshipType::HasMany,
            destination,
            "id",
            foreign_key,
        )
    }

    pub fn many_to_many(
        name: &str,
        destination: &str,
        join_resource: &str,
        this_side_key: &str,
        other_side_key: &str,
    ) -> Self {
        let mut relationship =
            Self::new(name, RelationshipType::ManyToMany, destination, "id", "id");
        relationship.join_resource = Some(join_resource.to_string());
        relationship.source_attribute = this_side_key.to_string();
        relationship.destination_attribute = other_side_key.to_string();
        relationship
    }

    /// The type the loaded relationship value takes on the record.
    /// Set the field type this relationship's loaded value takes.
    pub fn typed(mut self, ty: AttributeType) -> Self {
        self.attribute_type = ty;
        self
    }

    /// Accept the value as input on create and update actions.
    pub fn writable(mut self) -> Self {
        self.writable = true;
        self
    }

    pub fn not_public(mut self) -> Self {
        self.public = false;
        self
    }

    pub fn required(mut self) -> Self {
        self.allow_nil = false;
        self
    }

    pub fn describe(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Replace the default data layer loading with custom logic, the way Ash
    /// lets a relationship implement its own callbacks.
    pub fn with_loader<F, Fut>(mut self, loader: F) -> Self
    where
        F: Fn(Record) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<LoadedRelationship>> + Send + 'static,
    {
        self.loader = Some(Arc::new(move |record| Box::pin(loader(record))));
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn relationship_type(&self) -> RelationshipType {
        self.ty
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn source_attribute(&self) -> &str {
        &self.source_attribute
    }

    pub fn destination_attribute(&self) -> &str {
        &self.destination_attribute
    }

    pub fn join_resource(&self) -> Option<&str> {
        self.join_resource.as_deref()
    }

    pub fn attribute_type(&self) -> &AttributeType {
        &self.attribute_type
    }

    pub fn is_writable(&self) -> bool {
        self.writable
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn allows_nil(&self) -> bool {
        self.allow_nil
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn has_custom_loader(&self) -> bool {
        self.loader.is_some()
    }

    pub fn custom_loader(&self) -> Option<&RelationshipLoader> {
        self.loader.as_ref()
    }

    /// The value on this record that the join is made from, for the given
    /// source record. Returns nothing when the value is nil, so the loader can
    /// skip the query entirely.
    pub fn join_key(&self, record: &Record) -> Result<Option<FieldValue>> {
        if self.ty.joins_from_source() {
            let value = record.get(&self.source_attribute).ok_or_else(|| {
                Error::changeset(format!(
                    "relationship `{}` needs attribute `{}` which is not selected",
                    self.name, self.source_attribute
                ))
            })?;
            if super::attribute::is_nil(&value) {
                return Ok(None);
            }
            return Ok(Some(value));
        }

        // The key lives on the source side, so it is this record's own primary
        // key. The column is the relationship's source attribute, which is how a
        // `has_one` names the side that holds the key.
        let primary_key = record
            .primary_key(self.source_attribute.as_str())
            .or_else(|| record.primary_key("id"))
            .ok_or_else(|| {
                Error::changeset(format!(
                    "relationship `{}` needs a primary key on the record to load",
                    self.name
                ))
            })?;
        Ok(Some(primary_key))
    }
}

pub type CalculationFn =
    Arc<dyn Fn(Record) -> BoxFuture<'static, Result<FieldValue>> + Send + Sync>;

/// A value computed from a record, not stored in a column.
///
/// This is Ash's `calculate`. The default data layer treats calculations as
/// opaque and calls the provided function, so a calculation can be pure
/// (`title_case(name)`) or reach out to a domain to load something else.
#[derive(Clone)]
pub struct Calculation {
    name: String,
    ty: AttributeType,
    public: bool,
    description: Option<String>,
    calculate: CalculationFn,
}

impl std::fmt::Debug for Calculation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Calculation")
            .field("name", &self.name)
            .field("type", &self.ty)
            .field("public", &self.public)
            .finish()
    }
}

impl Calculation {
    pub fn new<F, Fut>(name: &str, ty: AttributeType, calculate: F) -> Self
    where
        F: Fn(Record) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<FieldValue>> + Send + 'static,
    {
        let calculate: CalculationFn = Arc::new(move |record| Box::pin(calculate(record)));
        Self {
            name: name.to_string(),
            ty,
            public: true,
            description: None,
            calculate,
        }
    }

    pub fn describe(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn private(mut self) -> Self {
        self.public = false;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &AttributeType {
        &self.ty
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn calculate(
        &self,
        record: Record,
    ) -> Pin<Box<dyn Future<Output = Result<FieldValue>> + Send>> {
        (self.calculate)(record)
    }
}
