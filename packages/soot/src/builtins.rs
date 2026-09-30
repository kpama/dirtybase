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
    record::Record,
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
/// The future shape a [`crate::extension::SootExtension::after_write`] closure
/// returns. It is here rather than in `extension` so every closure-driven hook
/// in the crate names its future from the same place.
pub type RecordFuture<'a> = Pin<Box<dyn Future<Output = Result<Record>> + Send + 'a>>;

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

/// Ready-made [`crate::extension::SootExtension`]s.
///
/// Ash ships equivalents for the concerns that come up in most applications —
/// who performed the write, whether this actor may, and which tenant the row
/// belongs to. Each one here narrows itself to the resources that declare what
/// it needs, so registering one on a whole domain is enough.
pub mod extensions {
    use std::{
        collections::BTreeMap,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;

    use crate::{
        attribute::Attribute,
        changeset::{Changeset, ChangesetKind},
        context::ActionContext,
        error::{Error, Errors, Result},
        extension::SootExtension,
        query::Query,
        record::Record,
        resource::ResourceDef,
    };

    /// Stamps who performed a write, the way Ash's `managed_by` and
    /// `changed_by` do.
    ///
    /// The extension adds the two columns, so nothing has to declare them, and
    /// fills them from the context's actor: `created_by` on a create, and
    /// `updated_by` on every write. A write with no actor is left unstamped
    /// rather than rejected — a system writing on its own behalf is normal.
    pub struct StampActor {
        created_by: String,
        updated_by: String,
    }

    impl Default for StampActor {
        fn default() -> Self {
            Self::new("created_by", "updated_by")
        }
    }

    impl StampActor {
        pub fn new(created_by: &str, updated_by: &str) -> Self {
            Self {
                created_by: created_by.to_string(),
                updated_by: updated_by.to_string(),
            }
        }
    }

    #[async_trait]
    impl SootExtension for StampActor {
        fn name(&self) -> &str {
            "stamp_actor"
        }

        fn description(&self) -> Option<&str> {
            Some("stamps the acting actor onto every write")
        }

        fn applies_to(&self, resource: &ResourceDef) -> bool {
            // Stamping is defined by the columns being there, and `extend` is
            // what puts them there — so this reads true after an extension of any
            // kind, and false only for a resource whose attributes were removed
            // again. Testing for their *absence* instead would filter the
            // extension out of exactly the resources it had just added itself to.
            resource.has_attribute(&self.updated_by)
        }

        fn extend(&self, resource: &mut ResourceDef) {
            if !resource.has_attribute(&self.created_by) {
                resource.add_attribute(
                    Attribute::uuid(&self.created_by)
                        .optional()
                        .describe("Stamped by the StampActor extension"),
                );
            }
            if !resource.has_attribute(&self.updated_by) {
                resource.add_attribute(
                    Attribute::uuid(&self.updated_by)
                        .optional()
                        .describe("Stamped by the StampActor extension"),
                );
            }
        }

        async fn before_changeset(
            &self,
            mut changeset: Changeset,
            context: &dyn ActionContext,
        ) -> Result<Changeset> {
            let Some(actor_id) = context.actor().and_then(|actor| actor.id()) else {
                return Ok(changeset);
            };

            // `updated_by` is always the actor on the write. `created_by` is too,
            // unless the caller supplied it — which usually means a system
            // creating a record on someone's behalf — in which case the explicit
            // author wins.
            changeset.set(&self.updated_by, actor_id.clone())?;
            if changeset.kind() == ChangesetKind::Create && changeset.is_nil(&self.created_by) {
                changeset.set(&self.created_by, actor_id)?;
            }
            Ok(changeset)
        }
    }

    /// Refuses actions that do not pass a check, the way Ash's policies do.
    ///
    /// The check sees the changeset on a write and the query on a read, so one
    /// extension can gate both. A refusal is reported as a changeset error
    /// rather than a framework failure, since it is the caller's input that was
    /// turned down.
    pub struct Authorize {
        name: String,
        error_message: String,
        check: Box<dyn AuthorizeCheck>,
    }

    /// What an [`Authorize`] check is handed.
    ///
    /// A changeset and a query are different things, so the check gets the parts
    /// they have in common — the resource, the action type, the caller's
    /// context — and reaches for the values it needs through the two
    /// optional handles.
    pub struct AuthorizeInput<'a> {
        pub resource: &'a ResourceDef,
        pub action_type: &'a crate::action::ActionType,
        pub changeset: Option<&'a Changeset>,
        pub query: Option<&'a Query>,
        pub context: &'a dyn ActionContext,
    }

    /// The predicate behind an [`Authorize`].
    pub trait AuthorizeCheck: Send + Sync {
        fn check(&self, input: AuthorizeInput<'_>) -> std::result::Result<(), String>;
    }

    impl Authorize {
        /// An extension that refuses an action unless `check` passes.
        pub fn new<F>(name: &str, error_message: &str, check: F) -> Self
        where
            F: Fn(AuthorizeInput<'_>) -> std::result::Result<(), String> + Send + Sync + 'static,
        {
            Self {
                name: name.to_string(),
                error_message: error_message.to_string(),
                check: Box::new(ClosureAuthorize { check }),
            }
        }

        /// The error reported when the check turns an action down.
        pub fn on_failure(mut self, error_message: &str) -> Self {
            self.error_message = error_message.to_string();
            self
        }
    }

    struct ClosureAuthorize<F> {
        check: F,
    }

    impl<F> AuthorizeCheck for ClosureAuthorize<F>
    where
        F: Fn(AuthorizeInput<'_>) -> std::result::Result<(), String> + Send + Sync,
    {
        fn check(&self, input: AuthorizeInput<'_>) -> std::result::Result<(), String> {
            (self.check)(input)
        }
    }

    #[async_trait]
    impl SootExtension for Authorize {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> Option<&str> {
            Some(&self.error_message)
        }

        async fn before_query(&self, query: &mut Query, context: &dyn ActionContext) -> Result<()> {
            let input = AuthorizeInput {
                resource: query.resource(),
                action_type: &query.action().action_type(),
                changeset: None,
                query: Some(query),
                context,
            };
            match self.check.check(input) {
                Ok(()) => Ok(()),
                Err(reason) => Err(self.refusal(query.resource().name(), &reason)),
            }
        }

        async fn before_changeset(
            &self,
            changeset: Changeset,
            context: &dyn ActionContext,
        ) -> Result<Changeset> {
            let input = AuthorizeInput {
                resource: changeset.resource(),
                action_type: &changeset.action_type(),
                changeset: Some(&changeset),
                query: None,
                context,
            };
            match self.check.check(input) {
                Ok(()) => Ok(changeset),
                Err(reason) => Err(self.refusal(changeset.resource().name(), &reason)),
            }
        }
    }

    impl Authorize {
        /// One refusal, phrased the same way whichever side it came from.
        fn refusal(&self, resource: &str, reason: &str) -> Errors {
            let detail = if reason.is_empty() {
                self.error_message.clone()
            } else {
                format!("{}: {reason}", self.error_message)
            };
            Error::changeset(format!("`{resource}` is not allowed: {detail}")).into()
        }
    }

    /// Keeps every row of a resource inside the caller's tenant, and stamps the
    /// tenant onto anything created.
    ///
    /// A tenant filter added to every read is the whole point: a caller that
    /// forgets to scope a query should not see another tenant's rows. The
    /// filter is added as an `AND`, so it narrows whatever the action and the
    /// caller already asked for rather than replacing it.
    ///
    /// A request with no tenant is refused rather than allowed through
    /// unscoped, since an unscoped read across tenants is the failure this
    /// extension exists to prevent.
    pub struct FilterByTenant {
        attribute: String,
        nullable: bool,
    }

    impl Default for FilterByTenant {
        fn default() -> Self {
            Self::new("tenant_id")
        }
    }

    impl FilterByTenant {
        pub fn new(attribute: &str) -> Self {
            Self {
                attribute: attribute.to_string(),
                nullable: false,
            }
        }

        /// Allow an action with no tenant through, leaving the column nil.
        ///
        /// Right for a resource that is shared across tenants, or for rows that
        /// deliberately belong to none.
        pub fn allow_without_tenant(mut self) -> Self {
            self.nullable = true;
            self
        }
    }

    #[async_trait]
    impl SootExtension for FilterByTenant {
        fn name(&self) -> &str {
            "filter_by_tenant"
        }

        fn description(&self) -> Option<&str> {
            Some("scopes every action to the caller's tenant")
        }

        fn applies_to(&self, resource: &ResourceDef) -> bool {
            resource.has_attribute(&self.attribute)
        }

        fn extend(&self, resource: &mut ResourceDef) {
            if !resource.has_attribute(&self.attribute) {
                resource.add_attribute(
                    Attribute::uuid(&self.attribute)
                        .optional()
                        .describe("The tenant this row belongs to"),
                );
            }
        }

        async fn before_query(&self, query: &mut Query, context: &dyn ActionContext) -> Result<()> {
            match context.tenant() {
                Some(tenant) => {
                    query.filter_eq(&self.attribute, tenant);
                    Ok(())
                }
                None if self.nullable => Ok(()),
                None => Err(Error::changeset(format!(
                    "a tenant is required to read `{}`",
                    query.resource().name()
                ))
                .into()),
            }
        }

        async fn before_changeset(
            &self,
            mut changeset: Changeset,
            context: &dyn ActionContext,
        ) -> Result<Changeset> {
            let Some(tenant) = context.tenant() else {
                if self.nullable {
                    return Ok(changeset);
                }
                return Err(Error::changeset(format!(
                    "a tenant is required to write `{}`",
                    changeset.resource().name()
                ))
                .into());
            };

            if changeset.is_nil(&self.attribute) {
                changeset.set(&self.attribute, tenant)?;
            }
            Ok(changeset)
        }

        async fn after_write(&self, mut record: Record, changeset: &Changeset) -> Result<Record> {
            // The column was written, but the write-back read is built from the
            // resource's attributes, so put the value on the record directly
            // rather than re-reading it.
            if let Some(value) = changeset.effective(&self.attribute) {
                record.set(&self.attribute, value);
            }
            Ok(record)
        }
    }

    /// Counts how many times each hook ran, for a caller asserting that an
    /// extension actually fired.
    ///
    /// Not a behaviour extension — it does nothing except observe — but the
    /// alternative in a test is a shared `Arc<AtomicUsize>` threaded through
    /// three closures.
    ///
    /// Cloning shares the counts, so a caller can keep a handle to read them
    /// after handing the extension itself to a resource.
    #[derive(Clone)]
    pub struct Recorder {
        name: String,
        seen: Arc<Mutex<BTreeMap<&'static str, usize>>>,
    }

    impl Recorder {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                seen: Arc::new(Mutex::new(BTreeMap::new())),
            }
        }

        /// How many times `hook` ran, by name: `before_query`, `before_changeset`,
        /// `after_write`, `after_read`, `setup` or `teardown`.
        pub fn count(&self, hook: &str) -> usize {
            self.seen
                .lock()
                .ok()
                .and_then(|seen| seen.get(hook).copied())
                .unwrap_or(0)
        }

        /// Every hook that ran, with its count.
        pub fn counts(&self) -> BTreeMap<String, usize> {
            self.seen
                .lock()
                .map(|seen| seen.iter().map(|(k, v)| ((*k).to_string(), *v)).collect())
                .unwrap_or_default()
        }
    }

    #[async_trait]
    impl SootExtension for Recorder {
        fn name(&self) -> &str {
            &self.name
        }

        async fn setup(&self, _context: &dyn ActionContext) -> Result<()> {
            self.record("setup");
            Ok(())
        }

        async fn teardown(&self) -> Result<()> {
            self.record("teardown");
            Ok(())
        }

        async fn before_query(
            &self,
            query: &mut Query,
            _context: &dyn ActionContext,
        ) -> Result<()> {
            let _ = query;
            self.record("before_query");
            Ok(())
        }

        async fn before_changeset(
            &self,
            changeset: Changeset,
            _context: &dyn ActionContext,
        ) -> Result<Changeset> {
            self.record("before_changeset");
            Ok(changeset)
        }

        async fn after_write(&self, record: Record, _changeset: &Changeset) -> Result<Record> {
            self.record("after_write");
            Ok(record)
        }

        async fn after_read(&self, records: Vec<Record>, _query: &Query) -> Result<Vec<Record>> {
            self.record("after_read");
            Ok(records)
        }
    }

    impl Recorder {
        fn record(&self, hook: &'static str) {
            if let Ok(mut seen) = self.seen.lock() {
                *seen.entry(hook).or_insert(0) += 1;
            }
        }
    }
}
