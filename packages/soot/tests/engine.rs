//! End-to-end tests against a real in-memory SQLite database.
//!
//! soot builds all of its SQL at runtime from metadata, so nothing about a
//! resource's declaration is checked by the compiler. These tests are the only
//! thing standing between a plausible-looking declaration and a query that
//! fails at runtime, so they deliberately go through the real `Manager` rather
//! than a mock data layer.

use std::{collections::BTreeMap, sync::Arc};

use dirtybase_db::field_values::FieldValue;
use dirtybase_soot::error::Result as SootResult;
use dirtybase_soot::{builtins::preparations, prelude::*, schema};

type Params = BTreeMap<String, FieldValue>;

fn params(pairs: &[(&str, &str)]) -> Params {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), FieldValue::String((*value).to_string())))
        .collect()
}

/// A small blog domain: a `User`, `Post`s that belong to one, and `Comment`s on
/// the post. Enough shapes to cover generated primary keys, foreign keys,
/// `has_many` loading, soft delete, aggregates, calculations and a custom
/// action in one place.
fn blog_domain() -> Domain {
    let user = ResourceDef::new("User")
        .uuid_primary_key()
        .attribute(Attribute::string("name").required())
        .attribute(Attribute::string("email").required())
        .timestamps()
        .soft_deletable()
        .default_actions();

    let post = ResourceDef::new("Post")
        .uuid_primary_key()
        .attribute(Attribute::string("title").required())
        .attribute(Attribute::text("body").optional())
        .attribute(Attribute::enumeration("status", &["draft", "published"]))
        .attribute(
            Attribute::integer("views")
                .default(0)
                .constraints(Constraints::new().min(0.0)),
        )
        .relationship(Relationship::belongs_to("author", "User", "author_id"))
        .relationship(Relationship::has_many("comments", "Comment", "post_id"))
        .aggregate(Aggregate::count("comment_count", "id").over_relationship("comments"))
        .calculation(Calculation::new(
            "slug",
            AttributeType::String,
            |record: Record| async move {
                let title = record.get_str("title");
                Ok(FieldValue::String(title.to_lowercase().replace(' ', "-")))
            },
        ))
        .timestamps()
        .default_actions()
        .action(
            Action::update("publish")
                .accept(&["title", "status", "updated_at"])
                .change(changes::set_attribute("status", "published"))
                .change(changes::set_timestamp("updated_at"))
                .validate(validations::attribute_present("title")),
        )
        .action(Action::generic_running("slugify", |input| async move {
            let title = input.data().get_str("title");
            Ok(FieldValue::String(title.to_lowercase().replace(' ', "-")))
        }));

    let comment = ResourceDef::new("Comment")
        .uuid_primary_key()
        .attribute(Attribute::text("body").required())
        .relationship(Relationship::belongs_to("post", "Post", "post_id"))
        .timestamps()
        .default_actions();

    // Registration order matters: `Post` and `Comment` both carry foreign keys
    // into tables registered before them.
    Domain::new().add(user).add(post).add(comment)
}

struct Harness {
    domain: Arc<Domain>,
    data_layer: Arc<dyn DataLayer>,
    context: DefaultActionContext,
}

impl Harness {
    /// A domain whose tables have actually been created in a fresh database.
    async fn new() -> Self {
        let manager = dirtybase_db::connector::sqlite::make_sqlite_in_memory_manager().await;
        let domain = Arc::new(blog_domain());
        schema::create_all_tables(&manager, domain.resources(), Some(&domain))
            .await
            .expect("schema should be creatable from the domain declaration");
        let data_layer = Arc::new(RelationalDataLayer::with_domain(
            manager,
            Arc::clone(&domain),
        ));
        Self {
            domain,
            data_layer,
            context: DefaultActionContext::new(),
        }
    }

    async fn create(&self, resource: &str, values: Params) -> SootResult<Record> {
        self.domain
            .create(&self.data_layer, resource, None, values, &self.context)
            .await
    }

    async fn update(
        &self,
        resource: &str,
        action: Option<&str>,
        id: &str,
        values: Params,
    ) -> SootResult<Record> {
        self.domain
            .update(
                &self.data_layer,
                resource,
                action,
                id,
                values,
                &self.context,
            )
            .await
    }

    async fn user(&self, name: &str, email: &str) -> Record {
        self.create("User", params(&[("name", name), ("email", email)]))
            .await
            .expect("user should be created")
    }

    async fn post(&self, author: &Record, title: &str) -> Record {
        let author_id = author.get_str("id");
        self.create(
            "Post",
            params(&[("title", title), ("author_id", &author_id)]),
        )
        .await
        .expect("post should be created")
    }
}

#[tokio::test]
async fn create_writes_every_attribute_and_reads_it_back() {
    let harness = Harness::new().await;

    let user = harness.user("Ada", "ada@example.com").await;

    assert_eq!(user.get_str("name"), "Ada");
    assert_eq!(user.get_str("email"), "ada@example.com");
    // The primary key is generated by the application, not the database, since
    // a uuid key cannot be auto-incremented.
    assert!(!user.is_nil("id"), "id should be set");
    // Timestamps were declared, so the engine supplied both.
    assert!(
        matches!(user.get("created_at"), Some(FieldValue::DateTime(_))),
        "created_at should be set, got {:?}",
        user.get("created_at")
    );
    assert!(
        matches!(user.get("updated_at"), Some(FieldValue::DateTime(_))),
        "updated_at should be set, got {:?}",
        user.get("updated_at")
    );
}

#[tokio::test]
async fn declared_defaults_are_applied_and_unsupplied_attributes_stay_nil() {
    let harness = Harness::new().await;
    let author = harness.user("Grace", "grace@example.com").await;

    let post = harness.post(&author, "Compilers").await;

    // `views` was declared with a default and the caller did not supply it.
    assert_eq!(post.get_i64("views"), Some(0));
    // `status` is an enumeration with no default, so it stays nil.
    assert!(post.is_nil("status"), "status should stay nil");
}

#[tokio::test]
async fn a_missing_required_attribute_is_reported_rather_than_written() {
    let harness = Harness::new().await;

    let error = harness
        .create("User", params(&[("name", "Nameless")]))
        .await
        .expect_err("a user with no email should be rejected");

    let rendered = format!("{error}");
    assert!(
        rendered.contains("email"),
        "the error should name the missing field, got: {rendered}"
    );
}

#[tokio::test]
async fn updates_only_touch_the_supplied_attributes() {
    let harness = Harness::new().await;
    let user = harness.user("Alan", "alan@example.com").await;
    let id = user.get_str("id");

    let updated = harness
        .update("User", None, &id, params(&[("name", "Alan Turing")]))
        .await
        .expect("user should be updated");

    assert_eq!(updated.get_str("name"), "Alan Turing");
    // An attribute the caller did not mention survives untouched.
    assert_eq!(updated.get_str("email"), "alan@example.com");
    assert_eq!(updated.get_str("id"), id);
}

#[tokio::test]
async fn a_custom_update_action_runs_its_changes() {
    let harness = Harness::new().await;
    let author = harness.user("Barbara", "barbara@example.com").await;
    let post = harness.post(&author, "On Compilers").await;
    let id = post.get_str("id");

    assert!(post.is_nil("status"), "the post starts with no status");

    let published = harness
        .update("Post", Some("publish"), &id, Params::new())
        .await
        .expect("the publish action should run");

    assert_eq!(published.get_str("status"), "published");
}

#[tokio::test]
async fn a_generic_action_on_a_record_sees_that_record() {
    let harness = Harness::new().await;
    let author = harness.user("Edsger", "edsger@example.com").await;
    let post = harness.post(&author, "On Proofs").await;
    let id = post.get_str("id");

    let slug = harness
        .domain
        .run_generic_on(
            &harness.data_layer,
            "Post",
            "slugify",
            &id,
            Params::new(),
            &harness.context,
        )
        .await
        .expect("the generic action should run");

    assert_eq!(slug.to_string(), "on-proofs");
}

#[tokio::test]
async fn a_calculation_is_computed_from_the_record() {
    let harness = Harness::new().await;
    let author = harness.user("Donald", "donald@example.com").await;

    let records = harness
        .domain
        .read(
            &harness.data_layer,
            "Post",
            None,
            |query| {
                query.calculate("slug");
            },
            &harness.context,
        )
        .await
        .expect("read should run");

    // Nothing was created, so the read is empty; the point is that asking for a
    // calculation the resource declared does not fail.
    assert!(records.is_empty());

    harness.post(&author, "Numerical Recipes").await;
    let records = harness
        .domain
        .read(
            &harness.data_layer,
            "Post",
            None,
            |query| {
                query.calculate("slug");
            },
            &harness.context,
        )
        .await
        .expect("read should run");
    assert_eq!(
        records[0].get("slug").unwrap().to_string(),
        "numerical-recipes"
    );
}

#[tokio::test]
async fn a_soft_deleted_row_is_hidden_by_default_but_still_reachable() {
    let harness = Harness::new().await;
    let user = harness.user("Ken", "ken@example.com").await;
    let id = user.get_str("id");

    harness
        .domain
        .destroy(&harness.data_layer, "User", None, &id, &harness.context)
        .await
        .expect("user should be destroyed");

    let live = harness
        .domain
        .read(&harness.data_layer, "User", None, |_| {}, &harness.context)
        .await
        .expect("read should run");
    assert!(
        live.is_empty(),
        "a soft-deleted row is not in the default result"
    );

    // The row is still there, so a soft-deletable resource stays auditable.
    let all = harness
        .domain
        .read(
            &harness.data_layer,
            "User",
            None,
            |query| {
                query.flag("include_deleted");
            },
            &harness.context,
        )
        .await
        .expect("read should run");
    assert_eq!(all.len(), 1, "asking explicitly for deleted rows finds it");
}

#[tokio::test]
async fn a_has_many_relationship_loads_onto_the_records() {
    let harness = Harness::new().await;
    let author = harness.user("Donald", "donald@example.com").await;
    let post = harness.post(&author, "Numerics").await;
    let post_id = post.get_str("id");

    for body in ["first", "second"] {
        harness
            .create("Comment", params(&[("body", body), ("post_id", &post_id)]))
            .await
            .expect("comment should be created");
    }

    let mut records = harness
        .domain
        .read(
            &harness.data_layer,
            "Post",
            None,
            |query| {
                query.filter_eq("id", &post_id);
            },
            &harness.context,
        )
        .await
        .expect("read should run");
    assert_eq!(records.len(), 1);

    // Loading is a separate step, so a caller can follow up on records it
    // already holds.
    harness
        .domain
        .load(&harness.data_layer, "Post", &mut records, &["comments"])
        .await
        .expect("comments should load");

    match records[0].loaded_relationship("comments") {
        Some(LoadedRelationship::ToMany(many)) => {
            assert_eq!(many.len(), 2, "both comments should come back");
        }
        other => panic!("expected a many relationship, got {other:?}"),
    }
}

#[tokio::test]
async fn a_belongs_to_relationship_loads_onto_the_record() {
    let harness = Harness::new().await;
    let author = harness.user("Radia", "radia@example.com").await;
    harness.post(&author, "Spanning Trees").await;

    let mut records = harness
        .domain
        .read(&harness.data_layer, "Post", None, |_| {}, &harness.context)
        .await
        .expect("read should run");
    harness
        .domain
        .load(&harness.data_layer, "Post", &mut records, &["author"])
        .await
        .expect("the author should load");

    match records[0].loaded_relationship("author") {
        Some(LoadedRelationship::ToOne(Some(one))) => {
            assert_eq!(one.get_str("name"), "Radia");
        }
        other => panic!("expected a to-one relationship holding a record, got {other:?}"),
    }
}

#[tokio::test]
async fn an_aggregate_over_a_relationship_counts_the_children() {
    let harness = Harness::new().await;
    let author = harness.user("Tony", "tony@example.com").await;
    let post = harness.post(&author, "Compilers").await;
    let post_id = post.get_str("id");

    for body in ["one", "two", "three"] {
        harness
            .create("Comment", params(&[("body", body), ("post_id", &post_id)]))
            .await
            .expect("comment should be created");
    }

    let records = harness
        .domain
        .read(
            &harness.data_layer,
            "Post",
            None,
            |query| {
                query.filter_eq("id", &post_id);
                query.aggregate(
                    Aggregate::count("comment_count", "id").over_relationship("comments"),
                );
            },
            &harness.context,
        )
        .await
        .expect("read should run");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].get_i64("comment_count"), Some(3));
}

#[tokio::test]
async fn a_foreign_key_to_a_missing_row_is_rejected_by_the_database() {
    let harness = Harness::new().await;

    let result = harness
        .create(
            "Post",
            params(&[
                ("title", "Orphan"),
                ("author_id", "00000000-0000-7000-8000-000000000000"),
            ]),
        )
        .await;

    assert!(
        result.is_err(),
        "the database should refuse a post whose author does not exist"
    );
}

#[tokio::test]
async fn an_attribute_constraint_is_checked_before_the_write() {
    let harness = Harness::new().await;
    let author = harness.user("Hedy", "hedy@example.com").await;
    let author_id = author.get_str("id");

    // `views` is constrained to be >= 0.
    let result = harness
        .create(
            "Post",
            params(&[
                ("title", "Negative"),
                ("author_id", &author_id),
                ("views", "-1"),
            ]),
        )
        .await;

    let error = result.expect_err("a negative `views` should be rejected");
    assert!(
        format!("{error}").contains("views"),
        "the error should name the attribute, got: {error}"
    );
}

#[tokio::test]
async fn filters_and_sorting_reach_the_database() {
    let harness = Harness::new().await;
    let author = harness.user("Linus", "linus@example.com").await;
    let author_id = author.get_str("id");

    for (title, status) in [("Aaa", "draft"), ("Bbb", "published"), ("Ccc", "published")] {
        harness
            .create(
                "Post",
                params(&[
                    ("title", title),
                    ("status", status),
                    ("author_id", &author_id),
                ]),
            )
            .await
            .expect("post should be created");
    }

    let records = harness
        .domain
        .read(
            &harness.data_layer,
            "Post",
            None,
            |query| {
                query.filter_eq("status", "published");
                query.sort_desc("title");
            },
            &harness.context,
        )
        .await
        .expect("query should run");

    let titles: Vec<String> = records
        .iter()
        .map(|record| record.get_str("title"))
        .collect();
    assert_eq!(titles, vec!["Ccc", "Bbb"], "filtered and sorted descending");
}

#[tokio::test]
async fn a_builtin_preparation_shapes_the_query() {
    let harness = Harness::new().await;
    let author = harness.user("Margaret", "margaret@example.com").await;
    let author_id = author.get_str("id");
    for title in ["One", "Two", "Three"] {
        harness
            .create(
                "Post",
                params(&[("title", title), ("author_id", &author_id)]),
            )
            .await
            .expect("post should be created");
    }

    // The same path a read action's preparation takes, applied by hand.
    let resource = Arc::clone(harness.domain.resource("Post").unwrap());
    let mut query = Query::new(Arc::clone(&resource), Arc::new(Action::read("read")));
    preparations::filter("title", FilterOperator::Eq, "Two")
        .prepare(&mut query, &harness.context)
        .await
        .expect("the builtin preparation should run");

    let records = harness
        .data_layer
        .read(resource, &query)
        .await
        .expect("query should run");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].get_str("title"), "Two");
}

#[tokio::test]
async fn a_read_action_runs_its_preparations_and_validations() {
    let harness = Harness::new().await;
    let author = harness.user("Katherine", "katherine@example.com").await;
    let author_id = author.get_str("id");
    for (title, status) in [("Kept", "draft"), ("Hidden", "published")] {
        harness
            .create(
                "Post",
                params(&[
                    ("title", title),
                    ("status", status),
                    ("author_id", &author_id),
                ]),
            )
            .await
            .expect("post should be created");
    }

    // Give `User` a read action that filters, so `read` has to run it.
    let domain = Domain::new()
        .add(blog_domain().resources()[0].clone().as_ref().clone())
        .add(
            ResourceDef::new("Post")
                .uuid_primary_key()
                .attribute(Attribute::string("title").required())
                .attribute(Attribute::enumeration("status", &["draft", "published"]))
                .attribute(Attribute::uuid("author_id").required())
                .relationship(Relationship::belongs_to("author", "User", "author_id"))
                .timestamps()
                .default_actions()
                .action(Action::read("drafts").prepare(preparations::filter(
                    "status",
                    FilterOperator::Eq,
                    "draft",
                ))),
        );

    let records = domain
        .read(
            &harness.data_layer,
            "Post",
            Some("drafts"),
            |_| {},
            &harness.context,
        )
        .await
        .expect("read should run");

    let titles: Vec<String> = records
        .iter()
        .map(|record| record.get_str("title"))
        .collect();
    assert_eq!(titles, vec!["Kept"], "the preparation filtered to drafts");
}

#[tokio::test]
async fn a_builtin_validation_runs_on_a_write() {
    let harness = Harness::new().await;

    let domain = Domain::new().add(
        ResourceDef::new("Widget")
            .uuid_primary_key()
            .attribute(Attribute::string("name").optional())
            .timestamps()
            .default_actions()
            .action(
                Action::create("strict")
                    .accept(&["name"])
                    .validate(validations::attribute_present("name")),
            ),
    );

    let error = domain
        .create(
            &harness.data_layer,
            "Widget",
            Some("strict"),
            Params::new(),
            &harness.context,
        )
        .await
        .expect_err("the strict action requires a name");

    assert!(
        format!("{error}").contains("name"),
        "the validation should name the field, got: {error}"
    );
}
