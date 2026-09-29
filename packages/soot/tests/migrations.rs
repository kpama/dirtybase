#![cfg(feature = "migrations")]

//! The `migrations` feature: a domain turned into real tables and back again.
//!
//! These run against SQLite like the engine tests, because the thing worth
//! checking is that the generated DDL is accepted by a database, not that the
//! blueprint looks right.
//!
//! They call [`SootMigration::create_tables`] rather than the `Migration` trait's
//! `up`, which is a thin adapter over exactly that. The trait needs a
//! `dirtybase_contract` app `Context`, and constructing one registers global
//! service resolvers in the process — the sort of thing that makes a test order
//! dependent. Testing the adapter as well would buy little for that cost.

use std::sync::Arc;

use dirtybase_db::field_values::FieldValue;
use dirtybase_soot::{
    builtins::changes,
    migrations::{SootMigration, TableOutcome},
    prelude::*,
};

/// The declaration, as a function so a `SootMigration` can rebuild it.
fn blog() -> Domain {
    let user = ResourceDef::new("User")
        .uuid_primary_key()
        .attribute(Attribute::string("email").required())
        .timestamps()
        .soft_deletable()
        .default_actions();

    let post = ResourceDef::new("Post")
        .uuid_primary_key()
        .attribute(Attribute::string("title").required())
        // Declares the `author_id` column and the foreign key onto it, without
        // the caller having to spell either out.
        .relationship(Relationship::belongs_to("author", "User", "author_id"))
        .relationship(Relationship::has_many("comments", "Comment", "post_id"))
        .timestamps()
        .default_actions();

    let comment = ResourceDef::new("Comment")
        .uuid_primary_key()
        .attribute(Attribute::text("body").required())
        .relationship(Relationship::belongs_to("post", "Post", "post_id"))
        .timestamps()
        .default_actions()
        // An action, to confirm actions do not affect the schema.
        .action(
            Action::create("draft")
                .accept(&["title"])
                .change(changes::set_attribute("title", "untitled")),
        );

    Domain::new().add(user).add(post).add(comment)
}

async fn manager() -> dirtybase_db::base::manager::Manager {
    dirtybase_db::connector::sqlite::make_sqlite_in_memory_manager().await
}

#[tokio::test]
async fn create_tables_creates_every_table_the_domain_declares() {
    let manager = manager().await;
    let migration = SootMigration::new(blog);

    let outcomes = migration
        .create_tables(&manager)
        .await
        .expect("the tables should be created");

    assert_eq!(
        outcomes,
        vec![
            TableOutcome {
                table: "User".into(),
                changed: true
            },
            TableOutcome {
                table: "Post".into(),
                changed: true
            },
            TableOutcome {
                table: "Comment".into(),
                changed: true
            },
        ],
        "every resource becomes a table, in registration order"
    );

    for table in ["User", "Post", "Comment"] {
        assert!(
            manager
                .has_table(table)
                .await
                .expect("has_table should work"),
            "`{table}` should exist"
        );
    }
}

#[tokio::test]
async fn the_generated_schema_enforces_the_implied_foreign_key() {
    let manager = manager().await;
    let domain = Arc::new(blog());
    SootMigration::new(blog)
        .create_tables(&manager)
        .await
        .expect("the tables should be created");

    // The point of `belongs_to` implying a column: `author_id` exists, and it is
    // a real foreign key the database enforces.
    let data_layer = RelationalDataLayer::with_domain(manager, Arc::clone(&domain));
    let result = data_layer
        .manager()
        .insert(
            "Post",
            [
                ("title", "Orphan"),
                ("author_id", "00000000-0000-7000-8000-000000000000"),
            ]
            .into_iter()
            .map(|(key, value)| (key.to_string(), FieldValue::String(value.to_string())))
            .collect::<std::collections::HashMap<String, FieldValue>>(),
        )
        .await;

    assert!(
        result.is_err(),
        "a post whose author does not exist should be refused"
    );
}

#[tokio::test]
async fn create_tables_is_idempotent_and_reports_what_it_skipped() {
    let manager = manager().await;
    let migration = SootMigration::new(blog);

    migration
        .create_tables(&manager)
        .await
        .expect("the first apply should work");

    let second = migration
        .create_tables(&manager)
        .await
        .expect("re-applying should be a no-op, not an error");

    assert!(
        second.iter().all(|outcome| !outcome.changed),
        "nothing should change on a second run, got {second:?}"
    );
}

#[tokio::test]
async fn drop_tables_removes_them_and_create_tables_brings_them_back() {
    let manager = manager().await;
    let migration = SootMigration::new(blog);

    migration
        .create_tables(&manager)
        .await
        .expect("the tables should be created");

    let dropped = migration
        .drop_tables(&manager)
        .await
        .expect("the tables should be dropped");
    // Reverse order, so a table is never dropped before the one pointing at it.
    assert_eq!(
        dropped.iter().map(|o| o.table.as_str()).collect::<Vec<_>>(),
        vec!["Comment", "Post", "User"]
    );
    for table in ["User", "Post", "Comment"] {
        assert!(
            !manager
                .has_table(table)
                .await
                .expect("has_table should work"),
            "`{table}` should be gone"
        );
    }

    migration
        .create_tables(&manager)
        .await
        .expect("re-creating after a drop should work");
    assert!(
        manager
            .has_table("Post")
            .await
            .expect("has_table should work")
    );
}

#[tokio::test]
async fn drop_tables_skips_a_table_that_was_never_created() {
    let manager = manager().await;
    let migration = SootMigration::new(blog);

    // Nothing has been created, so a drop has nothing to do and must not fail.
    let outcomes = migration
        .drop_tables(&manager)
        .await
        .expect("dropping nothing should succeed");
    assert!(
        outcomes.iter().all(|outcome| !outcome.changed),
        "nothing should be reported as dropped, got {outcomes:?}"
    );
}

#[tokio::test]
async fn only_creates_the_named_resources_in_the_order_given() {
    let manager = manager().await;

    // `Post` has a foreign key onto `User`, so asking for it alone would produce
    // DDL the database rejects. Asking for `User` alone is fine.
    SootMigration::new(blog)
        .only(["User"])
        .create_tables(&manager)
        .await
        .expect("a subset with no dangling foreign key should apply");

    assert!(
        manager
            .has_table("User")
            .await
            .expect("has_table should work")
    );
    assert!(
        !manager
            .has_table("Post")
            .await
            .expect("has_table should work"),
        "`posts` was not named, so it should not exist"
    );
}

#[tokio::test]
async fn tables_reports_the_mapping_without_a_database() {
    assert_eq!(
        SootMigration::new(blog).tables(),
        vec![
            ("User".to_string(), "User".to_string()),
            ("Post".to_string(), "Post".to_string()),
            ("Comment".to_string(), "Comment".to_string()),
        ]
    );
}

#[tokio::test]
async fn a_migrated_domain_is_immediately_usable_by_the_engine() {
    // The real risk with generated DDL is not that it looks right but that the
    // engine cannot then use it, so this goes all the way round: migrate, then
    // create, read, follow a relationship, and aggregate.
    let manager = manager().await;
    let domain = Arc::new(blog());
    SootMigration::new(blog)
        .create_tables(&manager)
        .await
        .expect("the tables should be created");

    let data_layer = RelationalDataLayer::with_domain(manager, Arc::clone(&domain));
    let data_layer: Arc<dyn DataLayer> = Arc::new(data_layer);
    let context = DefaultActionContext::new();
    let fields = |pairs: &[(&str, &str)]| -> std::collections::BTreeMap<String, FieldValue> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), FieldValue::String((*v).to_string())))
            .collect()
    };

    let user = domain
        .create(
            &data_layer,
            "User",
            None,
            fields(&[("email", "someone@example.com")]),
            &context,
        )
        .await
        .expect("a user should be creatable against the migrated schema");

    let post = domain
        .create(
            &data_layer,
            "Post",
            None,
            fields(&[("title", "Hello"), ("author_id", &user.get_str("id"))]),
            &context,
        )
        .await
        .expect("a post should be creatable against the migrated schema");

    for body in ["first", "second"] {
        domain
            .create(
                &data_layer,
                "Comment",
                None,
                fields(&[("body", body), ("post_id", &post.get_str("id"))]),
                &context,
            )
            .await
            .expect("a comment should be creatable");
    }

    // A foreign key written by the engine has to satisfy the constraint the
    // migration created.
    assert_eq!(post.get_str("author_id"), user.get_str("id"));

    let mut posts = domain
        .read(
            &data_layer,
            "Post",
            None,
            |query| {
                query.aggregate(
                    Aggregate::count("comment_count", "id").over_relationship("comments"),
                );
            },
            &context,
        )
        .await
        .expect("read should run");
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].get_i64("comment_count"), Some(2));

    domain
        .load(&data_layer, "Post", &mut posts, &["author"])
        .await
        .expect("the author should load");
    assert_eq!(
        posts[0]
            .loaded_relationship("author")
            .and_then(|loaded| loaded.as_to_one())
            .map(|author| author.get_str("email")),
        Some("someone@example.com".to_string())
    );
}
