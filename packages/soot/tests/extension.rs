//! Tests for extensions: what they may add to a declaration, and what they may
//! do while an action runs.
//!
//! The declaration half is pure metadata and needs no database. The hook half
//! runs against a real in-memory SQLite database, because the whole point of
//! these hooks is to affect what is persisted — an extension that filters a
//! query or stamps a column is only correct if the row it wrote is right.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use dirtybase_db::field_values::FieldValue;
use dirtybase_soot::{
    builtins::extensions::{Authorize, FilterByTenant, Recorder, StampActor},
    error::Result as SootResult,
    prelude::*,
    schema,
};

type Params = BTreeMap<String, FieldValue>;

fn params(pairs: &[(&str, &str)]) -> Params {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), FieldValue::String((*value).to_string())))
        .collect()
}

// ---- a domain with a couple of resources, and the database behind it --------

struct Harness {
    domain: Arc<Domain>,
    data_layer: Arc<dyn DataLayer>,
    context: DefaultActionContext,
}

impl Harness {
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

    async fn read(
        &self,
        resource: &str,
        configure: impl FnOnce(&mut Query),
    ) -> SootResult<Vec<Record>> {
        self.domain
            .read(&self.data_layer, resource, None, configure, &self.context)
            .await
    }
}

fn blog_domain() -> Domain {
    let user = ResourceDef::new("User")
        .uuid_primary_key()
        .attribute(Attribute::string("name").required())
        .timestamps()
        .default_actions();

    let post = ResourceDef::new("Post")
        .uuid_primary_key()
        .attribute(Attribute::string("title").required())
        .attribute(Attribute::uuid("updated_by").optional())
        .timestamps()
        .default_actions();

    Domain::new().add(user).add(post)
}

// ---- extensions used by more than one test ---------------------------------

/// Appends a fixed attribute value to every title, so a test can tell that the
/// hook ran without a database.
struct Suffix {
    suffix: String,
}

#[async_trait]
impl SootExtension for Suffix {
    fn name(&self) -> &str {
        "suffix"
    }

    fn description(&self) -> Option<&str> {
        Some("appends to the title on write")
    }

    async fn before_changeset(
        &self,
        mut changeset: Changeset,
        _context: &dyn ActionContext,
    ) -> SootResult<Changeset> {
        let title = changeset
            .attributes()
            .get("title")
            .cloned()
            .unwrap_or_default();
        changeset.set("title", format!("{title} {}", self.suffix))?;
        Ok(changeset)
    }
}

/// Counts the records it saw, so a test can assert a hook ran exactly once.
struct CountingRead {
    reads: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl SootExtension for CountingRead {
    fn name(&self) -> &str {
        "counting_read"
    }

    async fn after_read(&self, records: Vec<Record>, query: &Query) -> SootResult<Vec<Record>> {
        self.reads
            .lock()
            .expect("read log should not be poisoned")
            .push(query.resource().name().to_string());
        Ok(records)
    }
}

// ---- declaration: `extend` --------------------------------------------------

#[tokio::test]
async fn an_extension_can_add_attributes_to_the_resource_it_is_attached_to() {
    let post = ResourceDef::new("Post")
        .uuid_primary_key()
        .extension(StampActor::default())
        .default_actions();

    assert!(
        post.find_attribute("created_by").is_some(),
        "`extend` should have added the created_by column"
    );
    assert!(post.find_attribute("updated_by").is_some());

    // The column is not just declared, it is in the schema.
    let manager = dirtybase_db::connector::sqlite::make_sqlite_in_memory_manager().await;
    let domain = Domain::new().add(post);
    schema::create_all_tables(&manager, domain.resources(), Some(&domain))
        .await
        .expect("the extension's column should be creatable");
}

#[tokio::test]
async fn a_domain_extension_extends_every_resource_it_is_registered_on() {
    let domain = Domain::new()
        .add(
            ResourceDef::new("User")
                .uuid_primary_key()
                .default_actions(),
        )
        .add(
            ResourceDef::new("Post")
                .uuid_primary_key()
                .default_actions(),
        )
        .extension(FilterByTenant::default());

    for name in ["User", "Post"] {
        let resource = domain
            .resource(name)
            .expect("the resource should be registered");
        assert!(
            resource.find_attribute("tenant_id").is_some(),
            "`{name}` should have been extended"
        );
    }
}

#[tokio::test]
async fn a_resource_extension_does_not_reach_other_resources() {
    let domain = Domain::new()
        .add(
            ResourceDef::new("User")
                .uuid_primary_key()
                .default_actions()
                .extension(StampActor::default()),
        )
        .add(
            ResourceDef::new("Post")
                .uuid_primary_key()
                .default_actions(),
        );

    assert!(
        domain
            .resource("User")
            .expect("User should exist")
            .find_attribute("updated_by")
            .is_some()
    );
    assert!(
        domain
            .resource("Post")
            .expect("Post should exist")
            .find_attribute("updated_by")
            .is_none(),
        "a resource extension should stay on its own resource"
    );
}

// ---- resolution: which extensions run ---------------------------------------

#[test]
fn a_domain_extension_runs_before_a_resource_extension() {
    let domain = Domain::new()
        .add(
            ResourceDef::new("Post")
                .uuid_primary_key()
                .default_actions()
                .extension(StampActor::default()),
        )
        .extension(FilterByTenant::default());

    let post = domain.resource("Post").expect("Post should exist").clone();
    let names: Vec<String> = domain
        .extensions_of(&post)
        .iter()
        .map(|extension| extension.name().to_string())
        .collect();

    assert_eq!(names, vec!["filter_by_tenant", "stamp_actor"]);
}

#[test]
fn an_extension_that_does_not_apply_is_left_out() {
    /// Applies to a resource that has a `archived` attribute, and to no other.
    struct OnlyArchived;

    #[async_trait]
    impl SootExtension for OnlyArchived {
        fn name(&self) -> &str {
            "only_archived"
        }

        fn applies_to(&self, resource: &ResourceDef) -> bool {
            resource.has_attribute("archived")
        }
    }

    let domain = Domain::new()
        .add(
            ResourceDef::new("Post")
                .uuid_primary_key()
                .attribute(Attribute::boolean("archived"))
                .default_actions()
                .extension(OnlyArchived),
        )
        .add(
            ResourceDef::new("Page")
                .uuid_primary_key()
                .default_actions()
                .extension(OnlyArchived),
        );

    // Resolved extensions — what actually runs — respect `applies_to`, even
    // though both resources *registered* the extension.
    let resolved = |name: &str| -> Vec<String> {
        domain
            .extensions_of(
                &domain
                    .resource(name)
                    .expect("the resource should be registered"),
            )
            .iter()
            .map(|extension| extension.name().to_string())
            .collect()
    };

    assert_eq!(resolved("Post"), vec!["only_archived".to_string()]);
    assert!(
        resolved("Page").is_empty(),
        "Page has no `archived` attribute, so the extension is resolved away"
    );
}

#[test]
fn extensions_show_up_in_the_domain_description() {
    let domain = blog_domain().extension(FilterByTenant::default());

    let described = domain.describe();
    assert_eq!(described.extensions, &["filter_by_tenant".to_string()]);
}

// ---- hooks: what they do while an action runs -------------------------------

#[tokio::test]
async fn a_before_changeset_hook_can_change_what_is_written() {
    let harness = Harness::new().await;
    let domain = Domain::new()
        .add(
            ResourceDef::new("User")
                .uuid_primary_key()
                .attribute(Attribute::string("name").required())
                .timestamps()
                .default_actions(),
        )
        .add(
            ResourceDef::new("Post")
                .uuid_primary_key()
                .attribute(Attribute::string("title").required())
                .timestamps()
                .default_actions(),
        )
        .extension(Suffix {
            suffix: "!!".to_string(),
        });

    // The harness's tables were created from the undecorated domain, which is
    // the same shape, so the domain under test can use the same database.
    let data_layer = Arc::clone(&harness.data_layer);
    let context = DefaultActionContext::new();

    let record = domain
        .create(
            &data_layer,
            "Post",
            None,
            params(&[("title", "hello")]),
            &context,
        )
        .await
        .expect("the create should succeed");

    assert_eq!(record.get_str("title"), "hello !!");
}

#[tokio::test]
async fn a_before_query_hook_can_narrow_what_is_read() {
    let harness = Harness::new().await;
    harness
        .create("User", params(&[("name", "alice")]))
        .await
        .expect("alice should be created");
    harness
        .create("User", params(&[("name", "bob")]))
        .await
        .expect("bob should be created");

    let domain = Arc::new(blog_domain());
    let data_layer = Arc::clone(&harness.data_layer);
    let context = DefaultActionContext::new();

    let domain_with_filter =
        blog_domain().extension(extension("only_alice").before_query(|query, _context| {
            Box::pin(async move {
                query.filter_eq("name", "alice");
                Ok(())
            })
        }));
    let domain_with_filter = Arc::new(domain_with_filter);

    let everyone = domain
        .read(&data_layer, "User", None, |_| {}, &context)
        .await
        .expect("the read should succeed");
    assert_eq!(everyone.len(), 2, "without the hook both users are visible");

    let just_alice = domain_with_filter
        .read(&data_layer, "User", None, |_| {}, &context)
        .await
        .expect("the read should succeed");
    assert_eq!(just_alice.len(), 1);
    assert_eq!(just_alice[0].get_str("name"), "alice");
}

#[tokio::test]
async fn a_before_query_hook_can_refuse_the_action() {
    let harness = Harness::new().await;
    let context = DefaultActionContext::new();

    let domain =
        blog_domain().extension(extension("never_read").before_query(|_query, _context| {
            Box::pin(async move { Err(Error::changeset("reads are disabled").into()) })
        }));

    let result = domain
        .read(&harness.data_layer, "User", None, |_| {}, &context)
        .await;

    assert!(result.is_err(), "the hook should have refused the read");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("reads are disabled")
    );
}

#[tokio::test]
async fn an_after_read_hook_sees_the_matched_records() {
    let harness = Harness::new().await;
    harness
        .create("User", params(&[("name", "alice")]))
        .await
        .expect("the user should be created");

    let reads = Arc::new(Mutex::new(Vec::new()));
    let domain = blog_domain().extension(CountingRead {
        reads: Arc::clone(&reads),
    });

    let records = domain
        .read(
            &harness.data_layer,
            "User",
            None,
            |query| {
                query.filter_eq("name", "alice");
            },
            &DefaultActionContext::new(),
        )
        .await
        .expect("the read should succeed");

    assert_eq!(records.len(), 1);
    assert_eq!(
        reads
            .lock()
            .expect("the read log should not be poisoned")
            .as_slice(),
        &["User".to_string()],
        "the hook should have run once, for the one read"
    );
}

#[tokio::test]
async fn a_resource_hook_does_not_run_for_another_resource() {
    let harness = Harness::new().await;
    harness
        .create("User", params(&[("name", "alice")]))
        .await
        .expect("the user should be created");

    let reads = Arc::new(Mutex::new(Vec::new()));
    let domain = Domain::new()
        .add(
            ResourceDef::new("User")
                .uuid_primary_key()
                .attribute(Attribute::string("name").required())
                .timestamps()
                .default_actions()
                .extension(CountingRead {
                    reads: Arc::clone(&reads),
                }),
        )
        .add(
            ResourceDef::new("Post")
                .uuid_primary_key()
                .attribute(Attribute::string("title").required())
                .timestamps()
                .default_actions(),
        );

    domain
        .read(
            &harness.data_layer,
            "Post",
            None,
            |_| {},
            &DefaultActionContext::new(),
        )
        .await
        .expect("the read should succeed");

    assert!(
        reads
            .lock()
            .expect("the read log should not be poisoned")
            .is_empty(),
        "reading Post should not run User's hook"
    );
}

// ---- lifecycle --------------------------------------------------------------

/// Fails during setup, and records that teardown was reached.
struct FailsOnSetup {
    state: Arc<Mutex<Vec<&'static str>>>,
}

#[async_trait]
impl SootExtension for FailsOnSetup {
    fn name(&self) -> &str {
        "fails_on_setup"
    }

    async fn setup(&self, _context: &dyn ActionContext) -> SootResult<()> {
        self.state
            .lock()
            .expect("the state should not be poisoned")
            .push("setup");
        Err(Error::framework("setup failed").into())
    }

    async fn teardown(&self) -> SootResult<()> {
        self.state
            .lock()
            .expect("the state should not be poisoned")
            .push("teardown");
        Ok(())
    }
}

/// Records that its setup and teardown ran, in that order.
struct Lifecycle {
    state: Arc<Mutex<Vec<&'static str>>>,
}

#[async_trait]
impl SootExtension for Lifecycle {
    fn name(&self) -> &str {
        "lifecycle"
    }

    async fn setup(&self, _context: &dyn ActionContext) -> SootResult<()> {
        self.state
            .lock()
            .expect("the state should not be poisoned")
            .push("setup");
        Ok(())
    }

    async fn teardown(&self) -> SootResult<()> {
        self.state
            .lock()
            .expect("the state should not be poisoned")
            .push("teardown");
        Ok(())
    }
}

#[tokio::test]
async fn setup_and_teardown_reach_every_extension_in_the_domain() {
    let state = Arc::new(Mutex::new(Vec::new()));
    let domain = blog_domain()
        .extension(Lifecycle {
            state: Arc::clone(&state),
        })
        .extension(StampActor::default());
    let context = DefaultActionContext::new();

    domain
        .setup_extensions(&context)
        .await
        .expect("setup should succeed");

    domain
        .teardown_extensions()
        .await
        .expect("teardown should succeed");

    assert_eq!(
        *state.lock().expect("the state should not be poisoned"),
        vec!["setup", "teardown"]
    );
}

#[tokio::test]
async fn a_failing_setup_reports_the_error_and_keeps_going() {
    let state = Arc::new(Mutex::new(Vec::new()));
    let domain = blog_domain()
        .extension(FailsOnSetup {
            state: Arc::clone(&state),
        })
        .extension(Lifecycle {
            state: Arc::clone(&state),
        });

    let result = domain.setup_extensions(&DefaultActionContext::new()).await;

    let errors = result.expect_err("one extension failed, so setup should report it");
    assert_eq!(errors.len(), 1);
    assert!(errors.to_string().contains("setup failed"));

    assert_eq!(
        *state.lock().expect("the state should not be poisoned"),
        vec!["setup", "setup"],
        "the second extension should still have been set up"
    );
}

#[tokio::test]
async fn teardown_still_reaches_extensions_after_a_failing_setup() {
    let state = Arc::new(Mutex::new(Vec::new()));
    let domain = blog_domain().extension(FailsOnSetup {
        state: Arc::clone(&state),
    });

    let context = DefaultActionContext::new();
    let _ = domain.setup_extensions(&context).await;
    domain
        .teardown_extensions()
        .await
        .expect("teardown should succeed");

    assert_eq!(
        *state.lock().expect("the state should not be poisoned"),
        vec!["setup", "teardown"]
    );
}

// ---- builtins ---------------------------------------------------------------

#[tokio::test]
async fn the_tenant_filter_scopes_reads_and_stamps_writes() {
    let manager = dirtybase_db::connector::sqlite::make_sqlite_in_memory_manager().await;
    let domain = Arc::new(blog_domain().extension(FilterByTenant::default()));
    schema::create_all_tables(&manager, domain.resources(), Some(&domain))
        .await
        .expect("the tenant column should be creatable");

    let data_layer: Arc<dyn DataLayer> = Arc::new(RelationalDataLayer::with_domain(
        manager,
        Arc::clone(&domain),
    ));

    let tenant_a = DefaultActionContext::with_actor_and_tenant(
        Actor::from_id("01890a5d-ac96-774b-99b0-001d5d403bd1"),
        "01890a5e-0d70-7c64-b00e-2b0d5a4ff8e2",
    );
    let tenant_b = DefaultActionContext::with_actor_and_tenant(
        Actor::from_id("01890a5f-1e81-7d2f-a011-3c1e5f6a93f3"),
        "01890a60-2f92-7e30-b022-4d2f70718a04",
    );

    domain
        .create(
            &data_layer,
            "User",
            None,
            params(&[("name", "alice")]),
            &tenant_a,
        )
        .await
        .expect("alice should be created");

    let seen_by_a = domain
        .read(&data_layer, "User", None, |_| {}, &tenant_a)
        .await
        .expect("tenant a should see its own row");
    assert_eq!(seen_by_a.len(), 1);
    assert_eq!(
        seen_by_a[0].get_str("tenant_id"),
        "01890a5e-0d70-7c64-b00e-2b0d5a4ff8e2",
        "the write should have stamped the tenant"
    );

    let seen_by_b = domain
        .read(&data_layer, "User", None, |_| {}, &tenant_b)
        .await
        .expect("tenant b's read should succeed");
    assert!(
        seen_by_b.is_empty(),
        "tenant b should not see tenant a's rows"
    );
}

#[tokio::test]
async fn the_tenant_filter_refuses_an_action_with_no_tenant() {
    let manager = dirtybase_db::connector::sqlite::make_sqlite_in_memory_manager().await;
    let domain = Arc::new(blog_domain().extension(FilterByTenant::default()));
    schema::create_all_tables(&manager, domain.resources(), Some(&domain))
        .await
        .expect("the tenant column should be creatable");

    let data_layer: Arc<dyn DataLayer> = Arc::new(RelationalDataLayer::with_domain(
        manager,
        Arc::clone(&domain),
    ));

    let result = domain
        .create(
            &data_layer,
            "User",
            None,
            params(&[("name", "alice")]),
            &DefaultActionContext::new(),
        )
        .await;

    assert!(
        result.is_err(),
        "an unscoped write is the failure this extension exists to prevent"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("tenant is required")
    );
}

#[tokio::test]
async fn the_actor_stamp_records_who_wrote_a_row() {
    let manager = dirtybase_db::connector::sqlite::make_sqlite_in_memory_manager().await;
    let domain = Arc::new(
        blog_domain()
            .add(
                ResourceDef::new("Post")
                    .uuid_primary_key()
                    .attribute(Attribute::string("title").required())
                    .timestamps()
                    .default_actions(),
            )
            .extension(StampActor::default()),
    );
    schema::create_all_tables(&manager, domain.resources(), Some(&domain))
        .await
        .expect("the stamp columns should be creatable");

    let data_layer: Arc<dyn DataLayer> = Arc::new(RelationalDataLayer::with_domain(
        manager,
        Arc::clone(&domain),
    ));
    let context =
        DefaultActionContext::with_actor(Actor::from_id("01890a61-40a3-7f41-c033-5e3072825b15"));

    let record = domain
        .create(
            &data_layer,
            "Post",
            None,
            params(&[("title", "hello")]),
            &context,
        )
        .await
        .expect("the create should succeed");

    assert_eq!(
        record.get_str("created_by"),
        "01890a61-40a3-7f41-c033-5e3072825b15"
    );
    assert_eq!(
        record.get_str("updated_by"),
        "01890a61-40a3-7f41-c033-5e3072825b15"
    );
}

#[tokio::test]
async fn the_actor_stamp_leaves_a_supplied_created_by_alone() {
    let manager = dirtybase_db::connector::sqlite::make_sqlite_in_memory_manager().await;
    let domain = Arc::new(
        blog_domain()
            .add(
                ResourceDef::new("Post")
                    .uuid_primary_key()
                    .attribute(Attribute::string("title").required())
                    .timestamps()
                    .default_actions(),
            )
            .extension(StampActor::default()),
    );
    schema::create_all_tables(&manager, domain.resources(), Some(&domain))
        .await
        .expect("the stamp columns should be creatable");

    let data_layer: Arc<dyn DataLayer> = Arc::new(RelationalDataLayer::with_domain(
        manager,
        Arc::clone(&domain),
    ));
    let context =
        DefaultActionContext::with_actor(Actor::from_id("01890a61-40a3-7f41-c033-5e3072825b15"));

    // A system creating a record on someone's behalf passes the author through
    // explicitly, and the stamp should not overwrite it.
    let record = domain
        .create(
            &data_layer,
            "Post",
            None,
            params(&[
                ("title", "hello"),
                ("created_by", "01890a62-51b4-7052-d044-6f4183936c26"),
            ]),
            &context,
        )
        .await
        .expect("the create should succeed");

    assert_eq!(
        record.get_str("created_by"),
        "01890a62-51b4-7052-d044-6f4183936c26"
    );
    assert_eq!(
        record.get_str("updated_by"),
        "01890a61-40a3-7f41-c033-5e3072825b15"
    );
}

#[tokio::test]
async fn an_authorize_extension_can_refuse_a_write() {
    let manager = dirtybase_db::connector::sqlite::make_sqlite_in_memory_manager().await;
    let domain = Arc::new(blog_domain().extension(Authorize::new(
        "not_bob",
        "only bob may write",
        |_input| Err("the actor is not bob".to_string()),
    )));
    schema::create_all_tables(&manager, domain.resources(), Some(&domain))
        .await
        .expect("the tables should be creatable");

    let data_layer: Arc<dyn DataLayer> = Arc::new(RelationalDataLayer::with_domain(
        manager,
        Arc::clone(&domain),
    ));

    let result = domain
        .create(
            &data_layer,
            "User",
            None,
            params(&[("name", "alice")]),
            &DefaultActionContext::new(),
        )
        .await;

    let errors = result.expect_err("the check should have refused the write");
    assert!(errors.to_string().contains("only bob may write"));
    assert!(errors.to_string().contains("the actor is not bob"));
}

#[tokio::test]
async fn the_recorder_reports_what_it_saw() {
    let harness = Harness::new().await;
    harness
        .create("User", params(&[("name", "alice")]))
        .await
        .expect("the user should be created");

    let recorder = Recorder::new("audit");
    let domain = blog_domain().extension(recorder.clone());

    domain
        .create(
            &harness.data_layer,
            "User",
            None,
            params(&[("name", "bob")]),
            &DefaultActionContext::new(),
        )
        .await
        .expect("the create should succeed");
    domain
        .read(
            &harness.data_layer,
            "User",
            None,
            |_| {},
            &DefaultActionContext::new(),
        )
        .await
        .expect("the read should succeed");

    let counts = recorder.counts();
    assert_eq!(counts.get("before_changeset"), Some(&1));
    assert_eq!(counts.get("after_read"), Some(&1));
}
