//! A migration that materialises a whole [`Domain`] into the database.
//!
//! soot never issues DDL implicitly. Declaring a resource is pure data, so
//! whether it becomes a table is a deployment decision, and this is where that
//! decision is expressed: register [`SootMigration`] like any other dirtybase
//! migration and the domain's resources become tables on `up`, and go away again
//! on `down`.
//!
//! The work is exposed as [`SootMigration::create_tables`] and
//! [`SootMigration::drop_tables`], which need nothing but a `Manager`. The
//! [`Migration`] implementation is a thin adapter over those, so a caller that
//! already has an app `Context` can use the migration registry while a script, a
//! test, or a build step can call the methods directly.

use std::sync::Arc;

use dirtybase_contract::{
    app_contract::Context,
    db_contract::{base::manager::Manager, migration::Migration},
};

use crate::{domain::Domain, resource::ResourceDef};

/// Creates the tables for every resource in a [`Domain`].
///
/// The domain is rebuilt from a closure rather than captured, because a
/// migration is a unit that can be constructed at any time — including while
/// listing pending migrations, long after the process that registered the domain
/// has moved on. Rebuilding also keeps the migration `Send + Sync`.
pub struct SootMigration {
    build: Box<dyn Fn() -> Domain + Send + Sync>,
    only: Option<Vec<String>>,
}

/// What a `create_tables` or `drop_tables` call did, per table.
///
/// A migration reports nothing, so this is the only way a caller learns which
/// tables were actually touched — which is exactly what a test and a `--dry-run`
/// both want.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableOutcome {
    pub table: String,
    pub changed: bool,
}

impl SootMigration {
    /// Create every table declared by `build`.
    pub fn new<F>(build: F) -> Self
    where
        F: Fn() -> Domain + Send + Sync + 'static,
    {
        Self {
            build: Box::new(build),
            only: None,
        }
    }

    /// Create tables only for the named resources.
    ///
    /// Useful when a domain is large and the resources being deployed are a
    /// known subset. The order of `names` is respected, since a table with a
    /// foreign key has to exist before the one pointing at it.
    pub fn only<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.only = Some(names.into_iter().map(Into::into).collect());
        self
    }

    /// The resource → table mapping, in the order tables would be created.
    pub fn tables(&self) -> Vec<(String, String)> {
        self.resources()
            .iter()
            .map(|resource| {
                (
                    resource.name().to_string(),
                    resource.table_name().to_string(),
                )
            })
            .collect()
    }

    /// Create the tables that do not exist yet.
    ///
    /// A table that is already there is left alone. That is what makes this safe
    /// to re-run, and what lets a domain share a table with something else that
    /// owns it.
    pub async fn create_tables(
        &self,
        manager: &Manager,
    ) -> crate::error::Result<Vec<TableOutcome>> {
        // The domain is built twice: once to order the resources, and once to
        // resolve foreign keys while building each blueprint. Caching it here
        // would be faster, but the closure is a user function and calling it
        // once is the more predictable contract.
        let domain = (self.build)();
        let mut outcomes = Vec::new();
        for resource in self.resources() {
            let table = resource.table_name().to_string();
            if manager
                .has_table(&table)
                .await
                .map_err(|error| inspect_error(&table, error))?
            {
                outcomes.push(TableOutcome {
                    table,
                    changed: false,
                });
                continue;
            }
            crate::schema::create_table_with_domain(manager, &resource, Some(&domain)).await?;
            outcomes.push(TableOutcome {
                table,
                changed: true,
            });
        }
        Ok(outcomes)
    }

    /// Drop the tables this migration would have created.
    ///
    /// Reverse order: a table referenced by a foreign key has to be dropped
    /// after the table that points at it.
    pub async fn drop_tables(&self, manager: &Manager) -> crate::error::Result<Vec<TableOutcome>> {
        let mut outcomes = Vec::new();
        for resource in self.resources().into_iter().rev() {
            let table = resource.table_name().to_string();
            if !manager
                .has_table(&table)
                .await
                .map_err(|error| inspect_error(&table, error))?
            {
                outcomes.push(TableOutcome {
                    table,
                    changed: false,
                });
                continue;
            }
            crate::schema::drop_table(manager, &resource).await?;
            outcomes.push(TableOutcome {
                table,
                changed: true,
            });
        }
        Ok(outcomes)
    }

    /// The resources this migration will touch, in creation order.
    fn resources(&self) -> Vec<Arc<ResourceDef>> {
        let domain = (self.build)();
        match &self.only {
            Some(names) => {
                // Preserve the caller's order rather than the domain's, since
                // the caller's order is the one that satisfies the foreign keys.
                names
                    .iter()
                    .filter_map(|name| {
                        domain
                            .resources()
                            .iter()
                            .find(|resource| resource.name() == name)
                            .cloned()
                    })
                    .collect()
            }
            None => domain.resources().to_vec(),
        }
    }
}

/// A failure to even ask whether a table exists is worth naming the table for,
/// since there is no other context at that point in the loop.
fn inspect_error(table: &str, error: impl std::fmt::Display) -> crate::error::Errors {
    crate::error::from_db(&format!("could not inspect table `{table}`"), error)
}

#[dirtybase_contract::async_trait]
impl Migration for SootMigration {
    async fn up(&self, manager: &Manager, _: &Context) -> Result<(), anyhow::Error> {
        self.create_tables(manager)
            .await
            .map(|_| ())
            .map_err(|errors| anyhow::anyhow!("soot could not create its tables: {errors}"))
    }

    async fn down(&self, manager: &Manager, _: &Context) -> Result<(), anyhow::Error> {
        self.drop_tables(manager)
            .await
            .map(|_| ())
            .map_err(|errors| anyhow::anyhow!("soot could not drop its tables: {errors}"))
    }
}
