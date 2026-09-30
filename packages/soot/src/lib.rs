//! Soot is an Ash-style resource framework in the shape of this workspace.
//!
//! The design goal is that a domain entity is *data*, not code. A resource is
//! declared as a value — attributes, relationships, calculations, actions,
//! pipelines — and one generic engine, [`Domain`], knows how to run any of it
//! without ever learning a resource's Rust type. That is what Ash does with
//! macros and protocols; soot does it with runtime metadata, so a resource can
//! be built from configuration or a database row if a caller wants.
//!
//! # The pieces
//!
//! - [`resource::ResourceDef`] — a declaration. Attributes and their types,
//!   relationships, calculations, aggregates, actions, and code interfaces.
//! - [`action::Action`] — something a caller can invoke: `create`, `update`,
//!   `destroy`, `read`, or `generic`.
//! - [`change::Change`], [`validation::Validation`], [`preparation::Preparation`]
//!   — the behaviour attached to an action, run in declaration order.
//! - [`changeset::Changeset`] — the data flowing through a write, and the errors
//!   it accumulated.
//! - [`query::Query`] — the data flowing through a read.
//! - [`data_layer::DataLayer`] — where a changeset is actually persisted. Soot
//!   ships [`data_layer::RelationalDataLayer`], built on `dirtybase_db`.
//! - [`domain::Domain`] — the registry of resources and the action engine.
//!
//! # Example
//!
//! ```
//! use dirtybase_soot::prelude::*;
//!
//! let post = ResourceDef::new("Post")
//!     .uuid_primary_key()
//!     .attribute(Attribute::string("title").required())
//!     .attribute(Attribute::text("body").optional())
//!     .attribute(Attribute::enumeration("status", &["draft", "published"]))
//!     .relationship(Relationship::belongs_to("author", "User", "author_id"))
//!     .relationship(Relationship::has_many("comments", "Comment", "post_id"))
//!     .timestamps()
//!     .default_actions()
//!     .action(
//!         Action::update("publish")
//!             .accept(&["status", "updated_at"])
//!             .change(changes::set_attribute("status", "published"))
//!             .change(changes::set_timestamp("updated_at"))
//!             .validate(validations::attribute_present("status")),
//!     );
//!
//! let domain = Domain::new().add(post);
//! assert!(domain.has_resource("Post"));
//! ```
//!
//! Nothing above has touched a database. Declaring a resource is pure data;
//! only [`data_layer::RelationalDataLayer`] and [`schema`] need a `Manager`.
//!
//! With the `migrations` feature, [`migrations::SootMigration`] turns a whole
//! [`Domain`] into tables and back.

pub mod action;
pub mod aggregate;
pub mod attribute;
pub mod change;
pub mod changeset;
pub mod context;
pub mod data_layer;
pub mod domain;
pub mod error;
pub mod extension;
pub mod generic;
pub mod pipeline;
pub mod preparation;
pub mod query;
pub mod record;
pub mod relationship;
pub mod resource;
pub mod resource_impl;
pub mod schema;
pub mod validation;

pub mod builtins;

#[cfg(feature = "extension")]
pub mod config;
#[cfg(feature = "extension")]
pub mod dirtybase_entry;

#[cfg(feature = "migrations")]
pub mod migrations;

pub use dirtybase_db;

/// The names most callers need, in one import.
pub mod prelude {
    pub use crate::{
        action::{Action, ActionType, Argument},
        aggregate::{Aggregate, AggregateKind},
        attribute::{Attribute, AttributeType, Constraints},
        builtins::{changes, validations},
        change::Change,
        changeset::{Changeset, ChangesetKind},
        context::{ActionContext, Actor, DefaultActionContext},
        data_layer::{DataLayer, RelationalDataLayer},
        domain::Domain,
        error::{Error, ErrorClass, ErrorList, Errors, Result},
        extension::{ExtensionRef, FnExtension, SootExtension, extension},
        pipeline::Pipeline,
        preparation::Preparation,
        query::{Filter, FilterOperator, Query, SortDirection},
        record::Record,
        relationship::{Calculation, LoadedRelationship, Relationship, RelationshipType},
        resource::{InterfaceDefinition, ResourceDef},
        resource_impl::Resource,
        validation::Validation,
    };
}
