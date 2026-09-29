use crate::{changeset::Changeset, context::ActionContext, error::Result};

/// A transformation applied to a changeset before it reaches the data layer.
///
/// This is the closest thing to the core of Ash. Every piece of create, update
/// and destroy behaviour is a change, and changes run in declaration order. A
/// change takes a changeset and returns the modified one, exactly like Ash's
/// `change(changeset, opts, context)`.
#[async_trait::async_trait]
pub trait Change: Send + Sync {
    /// Transform the changeset.
    async fn change(&self, changeset: Changeset, context: &dyn ActionContext) -> Result<Changeset>;

    /// A human readable summary, surfaced through introspection.
    fn describe(&self) -> Option<String> {
        None
    }

    /// Whether this change will do anything. A change that returns false is
    /// skipped, which lets callers avoid work they do not need.
    fn has_change(&self, _changeset: &Changeset) -> bool {
        true
    }
}

/// Lets a boxed change be handed to `Action::change` without unwrapping, which
/// is what makes the closure builders in [`crate::builtins`] usable as
/// first-class changes.
#[async_trait::async_trait]
impl Change for Box<dyn Change> {
    async fn change(&self, changeset: Changeset, context: &dyn ActionContext) -> Result<Changeset> {
        (**self).change(changeset, context).await
    }

    fn describe(&self) -> Option<String> {
        (**self).describe()
    }

    fn has_change(&self, changeset: &Changeset) -> bool {
        (**self).has_change(changeset)
    }
}
