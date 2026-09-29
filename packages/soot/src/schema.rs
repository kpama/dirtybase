use std::sync::Arc;

use dirtybase_db::base::{
    column::{ColumnBlueprint, ColumnType},
    manager::Manager,
    table::TableBlueprint,
};

use crate::{
    attribute::AttributeType,
    error::{Result, from_db},
    resource::ResourceDef,
};

/// Build the table blueprint for a resource's declaration.
///
/// This is the bridge between soot's resource metadata and dirtybase's schema
/// API. Everything a resource knows about itself — its attributes, their types,
/// their constraints, whether it carries timestamps or soft delete — is enough
/// to produce the table. That is what lets a resource exist without a
/// hand-written migration beside it.
///
/// Foreign keys are not declared here, because resolving one needs the
/// destination resource's table name, and the destination is only known once
/// every resource is registered. [`table_blueprint_with_domain`] adds them.
pub fn table_blueprint(resource: &ResourceDef) -> TableBlueprint {
    build(resource, None)
}

/// The same blueprint, with foreign keys resolved against the domain.
pub fn table_blueprint_with_domain(
    resource: &ResourceDef,
    domain: &crate::domain::Domain,
) -> TableBlueprint {
    build(resource, Some(domain))
}

fn build(resource: &ResourceDef, domain: Option<&crate::domain::Domain>) -> TableBlueprint {
    let mut table = TableBlueprint::new(resource.table_name());
    table.set_is_new(true);

    for attribute in resource.attributes() {
        table.columns.push(column_for(resource, attribute, domain));
    }

    if resource.has_timestamps() {
        table.timestamps();
    }

    if resource.is_soft_deletable() {
        table.soft_deletable();
    }

    table
}

/// One attribute as one column.
///
/// A primary key is special-cased: an integer key becomes an auto-incrementing
/// column so the database assigns it, while a uuid or ulid key is generated in
/// the application and therefore needs a unique, non-null column instead.
fn column_for(
    resource: &ResourceDef,
    attribute: &crate::attribute::Attribute,
    domain: Option<&crate::domain::Domain>,
) -> ColumnBlueprint {
    let name = attribute.column_name();

    let mut column = if attribute.is_primary_key() && attribute.ty() == &AttributeType::Integer {
        ColumnBlueprint::new(name, ColumnType::AutoIncrementId)
    } else {
        ColumnBlueprint::new(name, attribute.ty().to_column_type())
    };

    if attribute.is_primary_key() {
        column.set_as_primary();
        column.set_is_unique(true);
        column.set_is_nullable(false);
    } else {
        // Nullability is driven by the attribute's own constraint, not by its
        // type, so an optional uuid and a required uuid differ only here.
        column.set_is_nullable(attribute.allows_nil());
    }

    // An explicit default is declared on the column too, so the database applies
    // it for writes that do not go through a changeset.
    if let Some(default) = attribute.declared_default() {
        column.set_default_from(default);
    }

    // A belongs_to relationship is a real foreign key on this table, so the
    // database can enforce that the target row exists. A has_many keeps its key
    // on the other side and declares nothing here.
    if let (Some(domain), Some((destination, destination_attribute))) = (
        domain,
        resource.relationship_target_column(attribute.name()),
    ) && let Ok(destination) = domain.resource(destination)
        && let Some(target) = destination.find_attribute(destination_attribute)
    {
        // Cascade on delete follows the resource: a soft-deletable resource
        // keeps its rows, so the database must not cascade the delete.
        column.references(
            destination.table_name(),
            target.column_name(),
            !resource.is_soft_deletable(),
            false,
        );
    }

    column
}

/// Create a resource's table if it does not already exist.
pub async fn create_table(manager: &Manager, resource: &ResourceDef) -> Result<()> {
    create_table_with_domain(manager, resource, None).await
}

/// Create a resource's table, resolving foreign keys through the domain when one
/// is supplied.
pub async fn create_table_with_domain(
    manager: &Manager,
    resource: &ResourceDef,
    domain: Option<&crate::domain::Domain>,
) -> Result<()> {
    let name = resource.table_name().to_string();
    let blueprint = build(resource, domain);
    manager
        .create_table_schema(&name, move |existing| {
            *existing = blueprint;
        })
        .await
        .map_err(|e| from_db(&format!("could not create table `{name}`"), e))
}

/// Create every resource's table.
///
/// Resources are created in registration order, so a resource must be
/// registered after anything its foreign keys point at.
pub async fn create_all_tables(
    manager: &Manager,
    resources: &[Arc<ResourceDef>],
    domain: Option<&crate::domain::Domain>,
) -> Result<()> {
    for resource in resources {
        create_table_with_domain(manager, resource, domain).await?;
    }
    Ok(())
}

/// Drop a resource's table.
pub async fn drop_table(manager: &Manager, resource: &ResourceDef) -> Result<()> {
    let name = resource.table_name().to_string();
    manager
        .drop_table(&name)
        .await
        .map_err(|e| from_db(&format!("could not drop table `{name}`"), e))
}
