use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use dirtybase_db::field_values::FieldValue;

use crate::{
    action::Action,
    aggregate::Aggregate,
    attribute::is_nil,
    error::{Error, ErrorList, Result},
    resource::ResourceDef,
};

/// A query flag asking for soft-deleted records to be included.
///
/// Ash spells this `include_deleted?`. It is a flag rather than a filter because
/// it changes the data layer's whole read path, not one predicate.
pub const INCLUDE_DELETED_FLAG: &str = "include_deleted";

/// Sort direction for one column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }
}

/// How a filter compares a column to a value.
///
/// Every variant maps onto an `Operator` in dirtybase's `QueryBuilder`, so the
/// data layer never has to interpret anything beyond this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOperator {
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    NotIn,
    Like,
    IsNull,
    IsNotNull,
}

/// Whether a group of filters is joined with AND or OR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterJoin {
    And,
    Or,
}

/// One predicate in a read action's filter list.
#[derive(Debug, Clone)]
pub struct Filter {
    attribute: String,
    operator: FilterOperator,
    value: Option<FieldValue>,
    values: Option<Vec<FieldValue>>,
    join: Option<FilterJoin>,
}

impl Filter {
    pub fn new(attribute: &str, operator: FilterOperator, value: impl Into<FieldValue>) -> Self {
        Self {
            attribute: attribute.to_string(),
            operator,
            value: Some(value.into()),
            values: None,
            join: None,
        }
    }

    pub fn in_values(mut self, values: impl IntoIterator<Item = FieldValue>) -> Self {
        self.operator = FilterOperator::In;
        self.values = Some(values.into_iter().collect());
        self.value = None;
        self
    }

    pub fn not_in_values(mut self, values: impl IntoIterator<Item = FieldValue>) -> Self {
        self.operator = FilterOperator::NotIn;
        self.values = Some(values.into_iter().collect());
        self.value = None;
        self
    }

    pub fn is_null(mut self) -> Self {
        self.operator = FilterOperator::IsNull;
        self.value = None;
        self
    }

    pub fn is_not_null(mut self) -> Self {
        self.operator = FilterOperator::IsNotNull;
        self.value = None;
        self
    }

    /// Join this filter to the previous one with OR instead of AND.
    pub fn or(mut self) -> Self {
        self.join = Some(FilterJoin::Or);
        self
    }

    pub fn attribute(&self) -> &str {
        &self.attribute
    }

    pub fn operator(&self) -> FilterOperator {
        self.operator
    }

    pub fn value(&self) -> Option<&FieldValue> {
        self.value.as_ref()
    }

    pub fn values(&self) -> Option<&[FieldValue]> {
        self.values.as_deref()
    }

    pub fn join(&self) -> Option<FilterJoin> {
        self.join
    }
}

/// The read side of an action: what to fetch and what to bring back with it.
///
/// This is Ash's `Ash.Query`. It accumulates filters, sorting, paging, the set of
/// relationships to load, calculations and aggregates, plus a set of flag style
/// arguments such as `include_count` or `include_deleted`.
#[derive(Clone)]
pub struct Query {
    resource: Arc<ResourceDef>,
    action: Arc<Action>,
    filters: Vec<Filter>,
    sort: Vec<(String, SortDirection)>,
    limit: Option<usize>,
    offset: Option<usize>,
    load: BTreeSet<String>,
    select: BTreeSet<String>,
    calculations: BTreeSet<String>,
    aggregates: Vec<Aggregate>,
    arguments: BTreeMap<String, FieldValue>,
    context: BTreeMap<String, FieldValue>,
    flags: BTreeSet<String>,
    errors: ErrorList,
}

impl std::fmt::Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Query")
            .field("resource", &self.resource.name())
            .field("action", &self.action.name())
            .field("filters", &self.filters)
            .field("sort", &self.sort)
            .field("limit", &self.limit)
            .field("offset", &self.offset)
            .field("load", &self.load)
            .finish_non_exhaustive()
    }
}

impl Query {
    pub fn new(resource: Arc<ResourceDef>, action: Arc<Action>) -> Self {
        Self {
            resource,
            action,
            filters: Vec::new(),
            sort: Vec::new(),
            limit: None,
            offset: None,
            load: BTreeSet::new(),
            select: BTreeSet::new(),
            calculations: BTreeSet::new(),
            aggregates: Vec::new(),
            arguments: BTreeMap::new(),
            context: BTreeMap::new(),
            flags: BTreeSet::new(),
            errors: ErrorList::new(),
        }
    }

    pub fn resource(&self) -> &Arc<ResourceDef> {
        &self.resource
    }

    pub fn action(&self) -> &Arc<Action> {
        &self.action
    }

    // ---- filters -----------------------------------------------------------

    pub fn filter(&mut self, filter: Filter) -> &mut Self {
        self.filters.push(filter);
        self
    }

    /// Alias for [`Query::filter`], for callers that read better as "add".
    pub fn add_filter(&mut self, filter: Filter) -> &mut Self {
        self.filter(filter)
    }

    pub fn filter_eq(&mut self, attribute: &str, value: impl Into<FieldValue>) -> &mut Self {
        self.filter(Filter::new(attribute, FilterOperator::Eq, value))
    }

    pub fn filter_ne(&mut self, attribute: &str, value: impl Into<FieldValue>) -> &mut Self {
        self.filter(Filter::new(attribute, FilterOperator::NotEq, value))
    }

    pub fn filter_gt(&mut self, attribute: &str, value: impl Into<FieldValue>) -> &mut Self {
        self.filter(Filter::new(attribute, FilterOperator::Gt, value))
    }

    pub fn filter_gte(&mut self, attribute: &str, value: impl Into<FieldValue>) -> &mut Self {
        self.filter(Filter::new(attribute, FilterOperator::Gte, value))
    }

    pub fn filter_lt(&mut self, attribute: &str, value: impl Into<FieldValue>) -> &mut Self {
        self.filter(Filter::new(attribute, FilterOperator::Lt, value))
    }

    pub fn filter_lte(&mut self, attribute: &str, value: impl Into<FieldValue>) -> &mut Self {
        self.filter(Filter::new(attribute, FilterOperator::Lte, value))
    }

    pub fn filter_in(
        &mut self,
        attribute: &str,
        values: impl IntoIterator<Item = FieldValue>,
    ) -> &mut Self {
        self.filter(Filter::new(attribute, FilterOperator::Eq, FieldValue::Null).in_values(values))
    }

    pub fn filter_like(&mut self, attribute: &str, pattern: &str) -> &mut Self {
        self.filter(Filter::new(attribute, FilterOperator::Like, pattern))
    }

    pub fn filter_is_null(&mut self, attribute: &str) -> &mut Self {
        self.filter(Filter::new(attribute, FilterOperator::IsNull, FieldValue::Null).is_null())
    }

    pub fn filter_is_not_null(&mut self, attribute: &str) -> &mut Self {
        self.filter(
            Filter::new(attribute, FilterOperator::IsNotNull, FieldValue::Null).is_not_null(),
        )
    }

    /// Match a `null` value explicitly, which is how a caller asks for records
    /// where an optional attribute was never set.
    pub fn filter_nil(&mut self, attribute: &str) -> &mut Self {
        let filter = if self.resource.is_soft_deletable()
            && attribute == self.resource.deleted_at_column()
        {
            Filter::new(attribute, FilterOperator::IsNull, FieldValue::Null).is_null()
        } else {
            Filter::new(
                attribute,
                FilterOperator::Eq,
                FieldValue::String(String::new()),
            )
        };
        self.filter(filter)
    }

    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    // ---- ordering and paging ----------------------------------------------

    pub fn sort_by(&mut self, attribute: &str, direction: SortDirection) -> &mut Self {
        self.sort.push((attribute.to_string(), direction));
        self
    }

    pub fn sort_asc(&mut self, attribute: &str) -> &mut Self {
        self.sort_by(attribute, SortDirection::Ascending)
    }

    pub fn sort_desc(&mut self, attribute: &str) -> &mut Self {
        self.sort_by(attribute, SortDirection::Descending)
    }

    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(&mut self, offset: usize) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    pub fn sorting(&self) -> &[(String, SortDirection)] {
        &self.sort
    }

    pub fn limit_by(&self) -> Option<usize> {
        self.limit
    }

    pub fn offset_by(&self) -> Option<usize> {
        self.offset
    }

    // ---- loading -----------------------------------------------------------

    /// Ask for a relationship to be loaded onto the returned records.
    pub fn load(&mut self, relationship: &str) -> &mut Self {
        self.load.insert(relationship.to_string());
        self
    }

    /// Ask for several relationships at once.
    pub fn load_many(&mut self, relationships: &[&str]) -> &mut Self {
        for relationship in relationships {
            self.load.insert(relationship.to_string());
        }
        self
    }

    pub fn load_all(&mut self) -> &mut Self {
        for relationship in self.resource.relationships() {
            if relationship.is_public() {
                self.load.insert(relationship.name().to_string());
            }
        }
        self
    }

    pub fn loads(&self) -> &BTreeSet<String> {
        &self.load
    }

    /// Restrict the returned columns. An empty selection returns every column.
    pub fn select(&mut self, attributes: &[&str]) -> &mut Self {
        for attribute in attributes {
            self.select.insert(attribute.to_string());
        }
        self
    }

    pub fn selects(&self) -> &BTreeSet<String> {
        &self.select
    }

    /// Compute a declared calculation for every returned record.
    pub fn calculate(&mut self, calculation: &str) -> &mut Self {
        self.calculations.insert(calculation.to_string());
        self
    }

    pub fn calculate_all(&mut self) -> &mut Self {
        for calculation in self.resource.calculations() {
            if calculation.is_public() {
                self.calculations.insert(calculation.name().to_string());
            }
        }
        self
    }

    pub fn calculates(&self) -> &BTreeSet<String> {
        &self.calculations
    }

    /// Add an aggregate to compute across the whole result set.
    pub fn aggregate(&mut self, aggregate: Aggregate) -> &mut Self {
        self.aggregates.push(aggregate);
        self
    }

    pub fn aggregates(&self) -> &[Aggregate] {
        &self.aggregates
    }

    // ---- arguments, context, flags ----------------------------------------

    pub fn argument(&self, name: &str) -> Option<FieldValue> {
        self.arguments.get(name).cloned()
    }

    pub fn set_argument(&mut self, name: &str, value: impl Into<FieldValue>) {
        self.arguments.insert(name.to_string(), value.into());
    }

    pub fn arguments(&self) -> &BTreeMap<String, FieldValue> {
        &self.arguments
    }

    pub fn get_context(&self, key: &str) -> Option<FieldValue> {
        self.context.get(key).cloned()
    }

    pub fn set_context(&mut self, key: &str, value: impl Into<FieldValue>) {
        self.context.insert(key.to_string(), value.into());
    }

    /// A boolean flag. Ash's read actions use these for switches such as
    /// `include_count?` and `include_deleted?`.
    pub fn flag(&mut self, flag: &str) -> &mut Self {
        self.flags.insert(flag.to_string());
        self
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    pub fn flags(&self) -> &BTreeSet<String> {
        &self.flags
    }

    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &ErrorList {
        &self.errors
    }

    /// Resolve argument defaults and reject missing required arguments.
    pub fn resolve_arguments(&mut self, supplied: &BTreeMap<String, FieldValue>) -> Result<()> {
        let resolved = self
            .action
            .resolve_arguments(supplied)
            .map_err(ErrorList::from)?;
        self.arguments.extend(resolved);
        Ok(())
    }

    /// Reject filters, sorts and loads that reference undeclared things, and
    /// filters on attributes marked not filterable.
    ///
    /// Runs after preparations, as Ash does, so a preparation can add a filter
    /// and still have it checked.
    pub fn validate(&mut self) -> Result<()> {
        let mut errors = ErrorList::new();

        for filter in &self.filters {
            match self.resource.find_attribute(filter.attribute()) {
                Some(attribute) => {
                    if !attribute.is_filterable() {
                        errors.push(Error::invalid(
                            filter.attribute(),
                            "cannot be used as a filter",
                        ));
                    }
                    if let Some(ty) = self
                        .resource
                        .find_attribute(filter.attribute())
                        .map(|a| a.ty())
                        && let Some(value) = filter.value()
                    {
                        let coerced = ty.coerce(value.clone());
                        if is_nil(&coerced) && !attribute.allows_nil() {
                            // A nil filter value on a required attribute is a
                            // caller mistake worth surfacing.
                            errors.push(Error::invalid(
                                filter.attribute(),
                                "is required and cannot be filtered as null",
                            ));
                        }
                    }
                }
                None => {
                    if self
                        .resource
                        .find_relationship(filter.attribute())
                        .is_none()
                    {
                        errors.push(Error::unknown_field(
                            filter.attribute(),
                            format!(
                                "`{}` is not an attribute of `{}`",
                                filter.attribute(),
                                self.resource.name()
                            ),
                        ));
                    }
                }
            }
        }

        for (attribute, _) in &self.sort {
            match self.resource.find_attribute(attribute) {
                Some(found) if found.is_sortable() => {}
                Some(_) => errors.push(Error::invalid(attribute, "cannot be sorted on")),
                None => errors.push(Error::unknown_field(
                    attribute,
                    format!(
                        "`{attribute}` is not an attribute of `{}`",
                        self.resource.name()
                    ),
                )),
            }
        }

        for relationship in &self.load {
            if self.resource.find_relationship(relationship).is_none() {
                errors.push(Error::unknown_field(
                    relationship,
                    format!(
                        "`{relationship}` is not a relationship of `{}`",
                        self.resource.name()
                    ),
                ));
            }
        }

        for calculation in &self.calculations {
            if self.resource.find_calculation(calculation).is_none() {
                errors.push(Error::unknown_field(
                    calculation,
                    format!(
                        "`{calculation}` is not a calculation of `{}`",
                        self.resource.name()
                    ),
                ));
            }
        }

        for attribute in &self.select {
            if self.resource.find_attribute(attribute).is_none()
                && self.resource.find_relationship(attribute).is_none()
            {
                errors.push(Error::unknown_field(
                    attribute,
                    format!(
                        "`{attribute}` is not an attribute of `{}`",
                        self.resource.name()
                    ),
                ));
            }
        }

        errors.into_result()
    }
}
