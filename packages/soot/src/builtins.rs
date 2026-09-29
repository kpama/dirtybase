//! Ready-made changes, validations and preparations.
//!
//! Ash ships an equivalent set. These cover the cases that come up in almost
//! every resource, so a declaration can stay short; anything else is a closure
//! implementing [`crate::change::Change`] and friends.

use std::{collections::BTreeMap, future::Future, pin::Pin};

use dirtybase_db::{field_values::FieldValue, types::ArcUuid7};

use crate::{
    change::Change,
    changeset::Changeset,
    context::ActionContext,
    error::{Error, ErrorList, Result},
    preparation::Preparation,
    query::Query,
    validation::Validation,
};

/// A boxed future, used so a closure can borrow its inputs.
///
/// The builders below take closures that return one of these rather than a bare
/// `async` block. The reason is that a change receives `&Changeset` and
/// `&dyn ActionContext`, so its future borrows them; naming that lifetime
/// explicitly is what lets a plain `Fn` — not a `FnOnce` — produce it on every
/// call.
pub type ChangeFuture<'a> = Pin<Box<dyn Future<Output = Result<Changeset>> + Send + 'a>>;
pub type ValidationFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
pub type PreparationFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

/// Build a [`Change`] from a closure.
pub fn change<F>(f: F) -> Box<dyn Change>
where
    F: for<'a> Fn(Changeset, &'a dyn ActionContext) -> ChangeFuture<'a> + Send + Sync + 'static,
{
    Box::new(ClosureChange { f })
}

struct ClosureChange<F> {
    f: F,
}

#[async_trait::async_trait]
impl<F> Change for ClosureChange<F>
where
    F: for<'a> Fn(Changeset, &'a dyn ActionContext) -> ChangeFuture<'a> + Send + Sync,
{
    async fn change(&self, changeset: Changeset, context: &dyn ActionContext) -> Result<Changeset> {
        (self.f)(changeset, context).await
    }
}

/// Build a [`Validation`] from a closure.
pub fn validation<F>(f: F) -> Box<dyn Validation>
where
    F: for<'a> Fn(&'a Changeset, &'a dyn ActionContext) -> ValidationFuture<'a>
        + Send
        + Sync
        + 'static,
{
    Box::new(ClosureValidation { f })
}

struct ClosureValidation<F> {
    f: F,
}

#[async_trait::async_trait]
impl<F> Validation for ClosureValidation<F>
where
    F: for<'a> Fn(&'a Changeset, &'a dyn ActionContext) -> ValidationFuture<'a> + Send + Sync,
{
    async fn validate(&self, changeset: &Changeset, context: &dyn ActionContext) -> Result<()> {
        (self.f)(changeset, context).await
    }
}

/// Build a [`Preparation`] from a closure.
pub fn preparation<F>(f: F) -> Box<dyn Preparation>
where
    F: for<'a> Fn(&'a mut Query, &'a dyn ActionContext) -> PreparationFuture<'a>
        + Send
        + Sync
        + 'static,
{
    Box::new(ClosurePreparation { f })
}

struct ClosurePreparation<F> {
    f: F,
}

#[async_trait::async_trait]
impl<F> Preparation for ClosurePreparation<F>
where
    F: for<'a> Fn(&'a mut Query, &'a dyn ActionContext) -> PreparationFuture<'a> + Send + Sync,
{
    async fn prepare(&self, query: &mut Query, context: &dyn ActionContext) -> Result<()> {
        (self.f)(query, context).await
    }
}

/// Changes that write a value onto a changeset.
pub mod changes {
    use super::*;

    /// Set an attribute to a fixed value.
    pub fn set_attribute(name: &str, value: impl Into<FieldValue>) -> Box<dyn Change> {
        let name = name.to_string();
        let value = value.into();
        change(move |mut changeset, _| {
            let name = name.clone();
            let value = value.clone();
            Box::pin(async move {
                changeset.set(&name, value)?;
                Ok(changeset)
            })
        })
    }

    /// Copy one attribute onto another, the common "denormalise" case.
    pub fn copy_attribute(from: &str, to: &str) -> Box<dyn Change> {
        let from = from.to_string();
        let to = to.to_string();
        change(move |mut changeset, _| {
            let from = from.clone();
            let to = to.clone();
            Box::pin(async move {
                if let Some(value) = changeset.get(&from) {
                    changeset.set(&to, value)?;
                }
                Ok(changeset)
            })
        })
    }

    /// Set an attribute only if the caller did not supply it, so an explicit
    /// input always wins over a default.
    pub fn set_default(name: &str, value: impl Into<FieldValue>) -> Box<dyn Change> {
        let name = name.to_string();
        let value = value.into();
        change(move |mut changeset, _| {
            let name = name.clone();
            let value = value.clone();
            Box::pin(async move {
                if changeset.is_nil(&name) {
                    changeset.set(&name, value)?;
                }
                Ok(changeset)
            })
        })
    }

    /// Stamp a timestamp column with the current time.
    ///
    /// This is what a resource with `timestamps()` uses to keep `updated_at`
    /// honest without every action having to remember it.
    pub fn set_timestamp(name: &str) -> Box<dyn Change> {
        let name = name.to_string();
        change(move |mut changeset, _| {
            let name = name.clone();
            Box::pin(async move {
                changeset.set(&name, FieldValue::DateTime(chrono::Utc::now()))?;
                Ok(changeset)
            })
        })
    }

    /// Generate a fresh uuid v7 for an attribute.
    pub fn generate_uuid(name: &str) -> Box<dyn Change> {
        let name = name.to_string();
        change(move |mut changeset, _| {
            let name = name.clone();
            Box::pin(async move {
                changeset.set(&name, FieldValue::from(ArcUuid7::default()))?;
                Ok(changeset)
            })
        })
    }

    /// Set a foreign key from another attribute.
    pub fn set_foreign_key(foreign_key: &str, source: &str) -> Box<dyn Change> {
        let foreign_key = foreign_key.to_string();
        let source = source.to_string();
        change(move |mut changeset, _| {
            let foreign_key = foreign_key.clone();
            let source = source.clone();
            Box::pin(async move {
                if let Some(value) = changeset.get(&source) {
                    changeset.set(&foreign_key, value)?;
                }
                Ok(changeset)
            })
        })
    }

    /// Remove an attribute from the changeset entirely, so it is not written.
    pub fn delete_attribute(name: &str) -> Box<dyn Change> {
        let name = name.to_string();
        change(move |mut changeset, _| {
            let name = name.clone();
            Box::pin(async move {
                changeset.remove(&name);
                Ok(changeset)
            })
        })
    }
}

/// Checks that run against a changeset after its changes have settled.
pub mod validations {
    use super::*;

    /// Require a non-nil value for an attribute.
    pub fn attribute_present(name: &str) -> Box<dyn Validation> {
        let name = name.to_string();
        validation(move |changeset, _| {
            let name = name.clone();
            Box::pin(async move {
                if changeset.is_nil(&name) {
                    Err(ErrorList::from(Error::required(&name, "is required")).into())
                } else {
                    Ok(())
                }
            })
        })
    }

    /// Require a non-nil value for several attributes, reporting all of the
    /// missing ones rather than stopping at the first.
    pub fn attributes_present(names: &[&str]) -> Box<dyn Validation> {
        let names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        validation(move |changeset, _| {
            let names = names.clone();
            Box::pin(async move {
                let mut errors = ErrorList::new();
                for name in &names {
                    if changeset.is_nil(name) {
                        errors.push(Error::required(name, "is required"));
                    }
                }
                errors.into_result()
            })
        })
    }

    /// Require a numeric attribute to be at least `min`.
    pub fn attribute_min(name: &str, min: f64) -> Box<dyn Validation> {
        let name = name.to_string();
        validation(move |changeset, _| {
            let name = name.clone();
            let min = min;
            Box::pin(async move {
                match changeset.number(&name) {
                    Some(value) if value < min => Err(ErrorList::from(
                        Error::invalid(&name, "is less than the minimum").with_var("min", min),
                    )
                    .into()),
                    _ => Ok(()),
                }
            })
        })
    }

    /// Require a numeric attribute to be at most `max`.
    pub fn attribute_max(name: &str, max: f64) -> Box<dyn Validation> {
        let name = name.to_string();
        validation(move |changeset, _| {
            let name = name.clone();
            let max = max;
            Box::pin(async move {
                match changeset.number(&name) {
                    Some(value) if value > max => Err(ErrorList::from(
                        Error::invalid(&name, "is greater than the maximum").with_var("max", max),
                    )
                    .into()),
                    _ => Ok(()),
                }
            })
        })
    }

    /// Require a string attribute to be at least `min_length` characters.
    pub fn attribute_min_length(name: &str, min_length: usize) -> Box<dyn Validation> {
        let name = name.to_string();
        validation(move |changeset, _| {
            let name = name.clone();
            let min_length = min_length;
            Box::pin(async move {
                match changeset.text(&name) {
                    Some(value) if value.chars().count() < min_length => Err(ErrorList::from(
                        Error::invalid(&name, "is too short").with_var("min", min_length as i64),
                    )
                    .into()),
                    _ => Ok(()),
                }
            })
        })
    }

    /// Require a string attribute to match a pattern.
    pub fn attribute_matches(name: &str, pattern: &str) -> Box<dyn Validation> {
        let name = name.to_string();
        let pattern = pattern.to_string();
        validation(move |changeset, _| {
            let name = name.clone();
            let pattern = pattern.clone();
            Box::pin(async move {
                let Some(value) = changeset.text(&name) else {
                    return Ok(());
                };
                if pattern_matches(&pattern, &value) {
                    return Ok(());
                }
                Err(ErrorList::from(
                    Error::invalid(&name, "does not match the expected pattern")
                        .with_var("pattern", FieldValue::String(pattern)),
                )
                .into())
            })
        })
    }

    /// Require two attributes to hold the same value.
    pub fn attributes_match(left: &str, right: &str) -> Box<dyn Validation> {
        let left = left.to_string();
        let right = right.to_string();
        validation(move |changeset, _| {
            let left = left.clone();
            let right = right.clone();
            Box::pin(async move {
                if changeset.get(&left) == changeset.get(&right) {
                    return Ok(());
                }
                Err(ErrorList::from(
                    Error::invalid(&left, "does not match")
                        .with_var("other", FieldValue::String(right)),
                )
                .into())
            })
        })
    }

    /// Require a value to be one of a fixed set.
    pub fn attribute_in_set(name: &str, allowed: &[&str]) -> Box<dyn Validation> {
        let name = name.to_string();
        let allowed: Vec<String> = allowed.iter().map(|value| value.to_string()).collect();
        validation(move |changeset, _| {
            let name = name.clone();
            let allowed = allowed.clone();
            Box::pin(async move {
                let Some(value) = changeset.text(&name) else {
                    return Ok(());
                };
                if allowed.iter().any(|candidate| candidate == &value) {
                    return Ok(());
                }
                Err(
                    ErrorList::from(Error::invalid(&name, "is not an allowed value").with_var(
                        "allowed",
                        FieldValue::Array(allowed.into_iter().map(FieldValue::String).collect()),
                    ))
                    .into(),
                )
            })
        })
    }

    /// Reject input the action does not accept, rather than ignoring it, so a
    /// typo surfaces instead of being silently dropped.
    pub fn no_unexpected_attributes(allowed: &[&str]) -> Box<dyn Validation> {
        let allowed: Vec<String> = allowed.iter().map(|name| name.to_string()).collect();
        validation(move |changeset, _| {
            let allowed = allowed.clone();
            Box::pin(async move {
                let mut errors = ErrorList::new();
                for name in changeset.provided_attributes() {
                    if !allowed.iter().any(|candidate| candidate == &name) {
                        errors.push(Error::unknown_field(
                            &name,
                            format!("`{name}` is not accepted by this action"),
                        ));
                    }
                }
                errors.into_result()
            })
        })
    }

    /// Build a [`Validation`] from a plain function of the caller's values, for
    /// checks that need the supplied values but not the changeset itself.
    ///
    /// This does not go through the closure builder above, because the check
    /// borrows data it owns rather than data the engine lends it, so there is no
    /// future lifetime to express.
    pub fn values<F>(f: F) -> Box<dyn Validation>
    where
        F: Fn(&BTreeMap<String, FieldValue>) -> Result<()> + Send + Sync + 'static,
    {
        struct ValuesValidation<F> {
            check: F,
        }

        #[async_trait::async_trait]
        impl<F> Validation for ValuesValidation<F>
        where
            F: Fn(&BTreeMap<String, FieldValue>) -> Result<()> + Send + Sync,
        {
            async fn validate(
                &self,
                changeset: &Changeset,
                _context: &dyn ActionContext,
            ) -> Result<()> {
                (self.check)(&changeset.provided_values())
            }
        }

        Box::new(ValuesValidation { check: f })
    }
}

/// Preparations that shape a read query.
pub mod preparations {
    use super::*;
    use crate::query::{Filter, FilterOperator, SortDirection};

    /// Add a filter, which is the common read-side preparation.
    pub fn filter(
        attribute: &str,
        operator: FilterOperator,
        value: impl Into<FieldValue>,
    ) -> Box<dyn Preparation> {
        let attribute = attribute.to_string();
        let value = value.into();
        preparation(move |query, _| {
            let attribute = attribute.clone();
            let value = value.clone();
            Box::pin(async move {
                query.add_filter(Filter::new(&attribute, operator, value));
                Ok(())
            })
        })
    }

    /// Sort the results.
    pub fn sort(attribute: &str, descending: bool) -> Box<dyn Preparation> {
        let attribute = attribute.to_string();
        preparation(move |query, _| {
            let attribute = attribute.clone();
            let descending = descending;
            Box::pin(async move {
                query.sort_by(
                    &attribute,
                    if descending {
                        SortDirection::Descending
                    } else {
                        SortDirection::Ascending
                    },
                );
                Ok(())
            })
        })
    }

    /// Cap the number of records.
    pub fn limit(limit: usize) -> Box<dyn Preparation> {
        preparation(move |query, _| {
            let limit = limit;
            Box::pin(async move {
                query.limit(limit);
                Ok(())
            })
        })
    }

    /// Follow a relationship on every returned record.
    pub fn load(relationship: &str) -> Box<dyn Preparation> {
        let relationship = relationship.to_string();
        preparation(move |query, _| {
            let relationship = relationship.clone();
            Box::pin(async move {
                query.load(&relationship);
                Ok(())
            })
        })
    }
}

/// A minimal pattern matcher.
///
/// soot deliberately does not take a regex dependency for one check. The pattern
/// syntax is literal characters, `*` for any run, and `\d` for a single digit,
/// which covers the format constraints attributes actually use.
fn pattern_matches(pattern: &str, value: &str) -> bool {
    fn match_here(pattern: &[u8], value: &[u8]) -> bool {
        if pattern.is_empty() {
            return value.is_empty();
        }
        match pattern[0] {
            // `*` matches any run of characters, so try the rest of the pattern
            // against every suffix.
            b'*' => {
                if match_here(&pattern[1..], value) {
                    return true;
                }
                (0..value.len()).any(|i| match_here(&pattern[1..], &value[i + 1..]))
            }
            b'\\' if pattern.len() > 1 && pattern[1] == b'd' => {
                !value.is_empty()
                    && value[0].is_ascii_digit()
                    && match_here(&pattern[2..], &value[1..])
            }
            expected => {
                !value.is_empty() && value[0] == expected && match_here(&pattern[1..], &value[1..])
            }
        }
    }

    match_here(pattern.as_bytes(), value.as_bytes())
}
