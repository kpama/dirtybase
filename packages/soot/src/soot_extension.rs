//! Extensions: behaviour attached to a resource, or to every resource in a
//! domain, without the resource itself knowing about it.
//!
//! This is Ash's `extensions` list. An Ash resource writes
//! `extensions [AshPostgres.Extensions.Calcite, MyApp.Policy]` and gets back
//! extra attributes, extra actions, and hooks that run around every action —
//! while the resource body stays exactly as it was written. soot's version
//! keeps that shape but makes the extension a runtime value like everything
//! else here: it is held behind an `Arc` next to the resource's attributes
//! rather than spliced in by a macro at compile time.
//!
//! # Where an extension can be registered
//!
//! - [`crate::resource::ResourceDef::extension`] — applies to that resource
//!   only.
//! - [`crate::domain::Domain::extension`] — applies to every resource in the
//!   domain. The two sets are merged, domain first, and
//!   [`SootExtension::applies_to`] filters the result per resource.
//!
//! # The two kinds of contribution
//!
//! An extension can change a resource's *declaration* through
//! [`SootExtension::extend`], which runs when the resource is registered. That
//! is how an extension adds the columns or actions it needs.
//!
//! It can also change what happens while an action *runs*, through the `async`
//! hooks. Those are the ones that let an extension do something the declaration
//! cannot express: refuse an action, stamp a value from the caller, or shape a
//! query.
//!
//! # Example
//!
//! ```
//! # use dirtybase_soot::prelude::*;
//! # use async_trait::async_trait;
//! /// Stamps who performed a write, the way Ash's `managed_by` does.
//! pub struct StampActor;
//!
//! #[async_trait]
//! impl SootExtension for StampActor {
//!     fn name(&self) -> &str { "stamp_actor" }
//!
//!     fn extend(&self, resource: &mut ResourceDef) {
//!         resource.add_attribute(Attribute::uuid("updated_by").optional());
//!     }
//!
//!     async fn before_changeset(
//!         &self,
//!         mut changeset: Changeset,
//!         context: &dyn ActionContext,
//!     ) -> Result<Changeset> {
//!         if let Some(id) = context.actor().and_then(|actor| actor.id()) {
//!             changeset.set("updated_by", id)?;
//!         }
//!         Ok(changeset)
//!     }
//! }
//!
//! let post = ResourceDef::new("Post")
//!     .uuid_primary_key()
//!     .attribute(Attribute::string("title").required())
//!     .extension(StampActor)
//!     .default_actions();
//!
//! // The extension added the column, so the declaration knows about it.
//! assert!(post.find_attribute("updated_by").is_some());
//! ```

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    changeset::Changeset,
    context::ActionContext,
    error::{ErrorList, Result},
    query::Query,
    record::Record,
    resource::ResourceDef,
};

/// A registered extension.
///
/// Held behind an `Arc` so a `ResourceDef` and a `Domain` both stay cheap to
/// clone, and so an extension is one value no matter how many resources it is
/// attached to.
pub type ExtensionRef = Arc<dyn SootExtension>;

/// A unit of behaviour attached to a resource, or to a whole domain.
///
/// Every hook has a default that does nothing, so an extension only implements
/// the ones it cares about. The `async` ones are all fallible and return a
/// [`crate::error::Result`]: a hook that fails fails the action, which is what
/// makes an extension usable as a gate rather than only as decoration.
#[async_trait]
pub trait SootExtension: Send + Sync {
    /// The extension's name, and the key it is looked up by.
    ///
    /// Two extensions sharing a name is allowed, but a lookup returns the
    /// first, so names are worth keeping unique.
    fn name(&self) -> &str;

    /// What the extension does, for [`crate::domain::Domain::describe`].
    fn description(&self) -> Option<&str> {
        None
    }

    /// Whether this extension applies to `resource`.
    ///
    /// A domain-level extension sees every resource, which is rarely what an
    /// extension wants: stamping `tenant_id` only makes sense on a resource
    /// that declares it. Returning `false` here is how an extension narrows
    /// itself without every declaration having to opt in.
    fn applies_to(&self, resource: &ResourceDef) -> bool {
        let _ = resource;
        true
    }

    /// Contribute to the resource's declaration.
    ///
    /// Called once, when the resource is registered. This is where an extension
    /// adds the attributes, relationships, actions, calculations or pipelines it
    /// needs — the parts of a resource that are still data at that point.
    ///
    /// It runs for domain extensions too, at the moment the resource is pushed
    /// into the domain, so a resource registered before the extension is added
    /// will not see it.
    fn extend(&self, resource: &mut ResourceDef) {
        let _ = resource;
    }

    /// Called once at boot, after every extension has been registered.
    ///
    /// The place to warm a cache or open a connection, since it is the only
    /// hook guaranteed to run outside an action.
    async fn setup(&self, context: &dyn ActionContext) -> Result<()> {
        let _ = context;
        Ok(())
    }

    /// Called once at shutdown, in the reverse of nothing — the same order they
    /// were set up in, which is the only order a registry has.
    async fn teardown(&self) -> Result<()> {
        Ok(())
    }

    /// Runs after a read action's preparations and before the query is
    /// validated, so a bad change here is reported before it reaches SQL.
    async fn before_query(&self, query: &mut Query, context: &dyn ActionContext) -> Result<()> {
        let _ = (query, context);
        Ok(())
    }

    /// Runs before an action's changes, with the changeset as the action
    /// assembled it.
    ///
    /// Returning a changeset hands the modified one to the rest of the pipeline,
    /// so this is where an extension stamps a value the caller did not supply.
    async fn before_changeset(
        &self,
        changeset: Changeset,
        context: &dyn ActionContext,
    ) -> Result<Changeset> {
        let _ = context;
        Ok(changeset)
    }

    /// Runs after the data layer has written, with the record as stored.
    ///
    /// Returning a record hands the modified one back to the caller, so this is
    /// where derived data that is not a calculation can be attached.
    async fn after_write(&self, record: Record, changeset: &Changeset) -> Result<Record> {
        let _ = changeset;
        Ok(record)
    }

    /// Runs after a read has come back from the data layer, with every record
    /// the query matched.
    async fn after_read(&self, records: Vec<Record>, query: &Query) -> Result<Vec<Record>> {
        let _ = query;
        Ok(records)
    }
}

/// A `SootExtension` built from closures, for an extension that only needs one
/// or two hooks.
///
/// A struct implementing the trait directly is usually clearer, but it means
/// writing seven method bodies to change one thing. This covers the short case
/// without hiding the trait.
///
/// The closures return a boxed future — the same convention the `changes::*` and
/// `validations::*` builders use — because a stored closure has to be
/// `Send + Sync` and cannot name a future that borrows its argument:
///
/// ```
/// # use dirtybase_soot::prelude::*;
/// let only_created_by_me = extension("only_created_by_me")
///     .before_changeset(|mut changeset, _context| {
///         Box::pin(async move {
///             changeset.set("updated_by", "someone")?;
///             Ok(changeset)
///         })
///     });
///
/// let resource = ResourceDef::new("Post")
///     .uuid_primary_key()
///     .attribute(Attribute::uuid("updated_by").optional())
///     .extension(only_created_by_me)
///     .default_actions();
/// assert!(resource.find_attribute("updated_by").is_some());
/// ```
pub struct FnExtension {
    name: String,
    description: Option<String>,
    before_changeset_fn: Option<
        Box<
            dyn for<'a> Fn(Changeset, &'a dyn ActionContext) -> crate::builtins::ChangeFuture<'a>
                + Send
                + Sync,
        >,
    >,
    before_query_fn: Option<
        Box<
            dyn for<'a> Fn(
                    &'a mut Query,
                    &'a dyn ActionContext,
                ) -> crate::builtins::PreparationFuture<'a>
                + Send
                + Sync,
        >,
    >,
    after_write_fn: Option<
        Box<
            dyn for<'a> Fn(Record, &'a Changeset) -> crate::builtins::RecordFuture<'a>
                + Send
                + Sync,
        >,
    >,
}

impl FnExtension {
    pub fn describe(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// An extension that only overrides [`SootExtension::before_changeset`].
    pub fn before_changeset<F>(mut self, f: F) -> Self
    where
        F: for<'a> Fn(Changeset, &'a dyn ActionContext) -> crate::builtins::ChangeFuture<'a>
            + Send
            + Sync
            + 'static,
    {
        self.before_changeset_fn = Some(Box::new(f));
        self
    }

    /// An extension that only overrides [`SootExtension::before_query`].
    pub fn before_query<F>(mut self, f: F) -> Self
    where
        F: for<'a> Fn(
                &'a mut Query,
                &'a dyn ActionContext,
            ) -> crate::builtins::PreparationFuture<'a>
            + Send
            + Sync
            + 'static,
    {
        self.before_query_fn = Some(Box::new(f));
        self
    }

    /// An extension that only overrides [`SootExtension::after_write`].
    pub fn after_write<F>(mut self, f: F) -> Self
    where
        F: for<'a> Fn(Record, &'a Changeset) -> crate::builtins::RecordFuture<'a>
            + Send
            + Sync
            + 'static,
    {
        self.after_write_fn = Some(Box::new(f));
        self
    }
}

#[async_trait]
impl SootExtension for FnExtension {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    async fn before_query(&self, query: &mut Query, context: &dyn ActionContext) -> Result<()> {
        match &self.before_query_fn {
            Some(f) => f(query, context).await,
            None => Ok(()),
        }
    }

    async fn before_changeset(
        &self,
        changeset: Changeset,
        context: &dyn ActionContext,
    ) -> Result<Changeset> {
        match &self.before_changeset_fn {
            Some(f) => f(changeset, context).await,
            None => Ok(changeset),
        }
    }

    async fn after_write(&self, record: Record, changeset: &Changeset) -> Result<Record> {
        match &self.after_write_fn {
            Some(f) => f(record, changeset).await,
            None => Ok(record),
        }
    }
}

/// Build a [`FnExtension`].
pub fn extension(name: &str) -> FnExtension {
    FnExtension {
        name: name.to_string(),
        description: None,
        before_changeset_fn: None,
        before_query_fn: None,
        after_write_fn: None,
    }
}

// ---- running a set of extensions ------------------------------------------
//
// The engine never holds a single extension. It holds the merged set for the
// resource it is about to act on, and runs it in order. These functions are
// that loop, kept out of `Domain` so the engine and a caller invoking an action
// by hand go through exactly the same code.
//
// A hook that fails stops the run and its error is reported as it came: a
// changeset that a hook was going to modify no longer exists, so there is
// nothing to carry on with. That matches how a failing change is already
// treated, and it means an extension is a real gate rather than advice.

// ---- declaration time ------------------------------------------------------

/// Let each extension contribute to `resource`.
///
/// Applied by [`crate::domain::Domain::push`] for the domain's own extensions,
/// and by the resource builder for the ones registered on the resource itself.
pub fn extend_resource(extensions: &[ExtensionRef], resource: &mut ResourceDef) {
    for extension in extensions {
        extension.extend(resource);
    }
}

// ---- boot ------------------------------------------------------------------

/// Run [`SootExtension::setup`] on every extension, collecting failures.
///
/// Setup does not stop the first failure: a broken extension should not hide
/// the state of the ones after it, and the caller gets to see both.
pub async fn setup_all(extensions: &[ExtensionRef], context: &dyn ActionContext) -> Result<()> {
    let mut errors = ErrorList::new();
    for extension in extensions {
        errors.add_result(extension.setup(context).await);
    }
    errors.into_result()
}

/// Run [`SootExtension::teardown`] on every extension, collecting failures.
///
/// A teardown that fails is logged rather than propagated, because shutdown is
/// already the worst moment to raise a new error.
pub async fn teardown_all(extensions: &[ExtensionRef]) -> Result<()> {
    let mut errors = ErrorList::new();
    for extension in extensions {
        errors.add_result(extension.teardown().await);
    }
    errors.into_result()
}

// ---- action lifecycle ------------------------------------------------------

/// Run [`SootExtension::before_query`] on every extension, in order.
///
/// Each one sees the query as the previous left it, which is what lets a later
/// extension narrow a filter an earlier one added.
pub async fn before_query(
    extensions: &[ExtensionRef],
    query: &mut Query,
    context: &dyn ActionContext,
) -> Result<()> {
    for extension in extensions {
        extension.before_query(query, context).await?;
    }
    Ok(())
}

/// Run [`SootExtension::before_changeset`] on every extension, in order.
pub async fn before_changeset(
    extensions: &[ExtensionRef],
    mut changeset: Changeset,
    context: &dyn ActionContext,
) -> Result<Changeset> {
    for extension in extensions {
        changeset = extension.before_changeset(changeset, context).await?;
    }
    Ok(changeset)
}

/// Run [`SootExtension::after_write`] on every extension, in order.
pub async fn after_write(
    extensions: &[ExtensionRef],
    mut record: Record,
    changeset: &Changeset,
) -> Result<Record> {
    for extension in extensions {
        record = extension.after_write(record, changeset).await?;
    }
    Ok(record)
}

/// Run [`SootExtension::after_read`] on every extension, in order.
pub async fn after_read(
    extensions: &[ExtensionRef],
    mut records: Vec<Record>,
    query: &Query,
) -> Result<Vec<Record>> {
    for extension in extensions {
        records = extension.after_read(records, query).await?;
    }
    Ok(records)
}

/// The name of an extension, for [`crate::domain::Domain::describe`].
pub fn extension_names(extensions: &[ExtensionRef]) -> Vec<String> {
    extensions
        .iter()
        .map(|extension| extension.name().to_string())
        .collect()
}
