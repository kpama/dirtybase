use crate::{changeset::Changeset, context::ActionContext, error::Result};

/// A check that a changeset is internally consistent.
///
/// Validations run after changes, in declaration order, and never modify the
/// changeset: they only add errors.
#[async_trait::async_trait]
pub trait Validation: Send + Sync {
    async fn validate(&self, changeset: &Changeset, context: &dyn ActionContext) -> Result<()>;

    fn describe(&self) -> Option<String> {
        None
    }
}

/// Lets a boxed validation be handed to `Action::validate` directly.
#[async_trait::async_trait]
impl Validation for Box<dyn Validation> {
    async fn validate(&self, changeset: &Changeset, context: &dyn ActionContext) -> Result<()> {
        (**self).validate(changeset, context).await
    }

    fn describe(&self) -> Option<String> {
        (**self).describe()
    }
}
