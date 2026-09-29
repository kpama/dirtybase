use std::sync::Arc;

use dirtybase_db::field_values::FieldValue;

use crate::{
    change::Change, changeset::Changeset, preparation::Preparation, query::Query,
    validation::Validation,
};

/// A reusable group of changes, validations and preparations that several
/// actions can share, so behaviour is defined once rather than copied.
///
/// This is the soot equivalent of Ash's `pipelines` / `pipe_through`. Pipelines
/// are inlined at the point of reference, so a change declared before
/// `pipe_through` still runs first.
#[derive(Clone, Default)]
pub struct Pipeline {
    name: String,
    changes: Vec<PipelineEntity<dyn Change>>,
    validations: Vec<PipelineEntity<dyn Validation>>,
    preparations: Vec<PipelineEntity<dyn Preparation>>,
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("name", &self.name)
            .field("changes", &self.changes.len())
            .field("validations", &self.validations.len())
            .field("preparations", &self.preparations.len())
            .finish()
    }
}

impl Pipeline {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn change<C: Change + 'static>(mut self, change: C) -> Self {
        self.changes.push(PipelineEntity::always(Box::new(change)));
        self
    }

    pub fn validate<V: Validation + 'static>(mut self, validation: V) -> Self {
        self.validations
            .push(PipelineEntity::always(Box::new(validation)));
        self
    }

    pub fn prepare<P: Preparation + 'static>(mut self, preparation: P) -> Self {
        self.preparations
            .push(PipelineEntity::always(Box::new(preparation)));
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn changes(&self) -> &[PipelineEntity<dyn Change>] {
        &self.changes
    }

    pub fn validations(&self) -> &[PipelineEntity<dyn Validation>] {
        &self.validations
    }

    pub fn preparations(&self) -> &[PipelineEntity<dyn Preparation>] {
        &self.preparations
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty() && self.validations.is_empty() && self.preparations.is_empty()
    }
}

/// Decides whether a change, validation or preparation applies.
///
/// This is soot's `where` condition. When it evaluates to false the wrapped
/// entity is skipped entirely.
#[derive(Debug, Clone)]
pub enum Condition {
    /// Always true.
    Always,
    /// True when the named attribute holds a non-nil value.
    Present(String),
    /// True when the named attribute is absent or nil.
    Absent(String),
    /// True when the named attribute equals the given value.
    Equals(String, FieldValue),
    /// True when the named action argument equals the given value.
    ArgumentEquals(String, FieldValue),
}

impl Condition {
    pub fn always() -> Self {
        Self::Always
    }

    pub fn present(attribute: &str) -> Self {
        Self::Present(attribute.to_string())
    }

    pub fn absent(attribute: &str) -> Self {
        Self::Absent(attribute.to_string())
    }

    pub fn equals(attribute: &str, value: impl Into<FieldValue>) -> Self {
        Self::Equals(attribute.to_string(), value.into())
    }

    pub fn argument_equals(argument: &str, value: impl Into<FieldValue>) -> Self {
        Self::ArgumentEquals(argument.to_string(), value.into())
    }

    /// Whether the condition holds for a changeset.
    pub fn holds_for_changeset(&self, changeset: &Changeset) -> bool {
        match self {
            Self::Always => true,
            Self::Present(attribute) => !changeset.is_nil(attribute),
            Self::Absent(attribute) => changeset.is_nil(attribute),
            Self::Equals(attribute, value) => changeset.get(attribute).as_ref() == Some(value),
            Self::ArgumentEquals(argument, value) => {
                changeset.argument(argument).as_ref() == Some(value)
            }
        }
    }

    /// Whether the condition holds for a query.
    ///
    /// Attribute conditions are not meaningful against a query, which carries
    /// arguments and filters rather than values, so they are treated as
    /// satisfied. Argument conditions do apply.
    pub fn holds_for_query(&self, query: &Query) -> bool {
        match self {
            Self::Always => true,
            Self::Present(_) | Self::Absent(_) | Self::Equals(..) => true,
            Self::ArgumentEquals(argument, value) => {
                query.argument(argument).as_ref() == Some(value)
            }
        }
    }
}

/// A change, validation or preparation paired with the condition gating it.
///
/// Held behind an `Arc` so a `ResourceDef` stays cheap to share.
pub struct PipelineEntity<T: ?Sized> {
    inner: Arc<T>,
    condition: Condition,
}

impl<T: ?Sized> Clone for PipelineEntity<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            condition: self.condition.clone(),
        }
    }
}

impl<T: ?Sized> std::fmt::Debug for PipelineEntity<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineEntity")
            .field("condition", &self.condition)
            .finish_non_exhaustive()
    }
}

impl<T: ?Sized> PipelineEntity<T> {
    pub fn always(inner: Box<T>) -> Self {
        Self {
            inner: Arc::from(inner),
            condition: Condition::Always,
        }
    }

    pub fn when(inner: Box<T>, condition: Condition) -> Self {
        Self {
            inner: Arc::from(inner),
            condition,
        }
    }

    pub fn condition(&self) -> &Condition {
        &self.condition
    }

    pub fn get(&self) -> &T {
        &self.inner
    }
}
