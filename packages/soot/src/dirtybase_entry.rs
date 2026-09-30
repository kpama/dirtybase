//! soot as a dirtybase extension.
//!
//! The engine in this crate needs nothing but a `Manager`, so on its own soot is
//! a library. This module is the piece that plugs it into a dirtybase
//! application: register [`Extension`] and the application's domain becomes
//! reachable from any request or command through `context.get::<Soot>()`.
//!
//! ```ignore
//! use dirtybase_soot::dirtybase_entry::{Extension, Soot};
//!
//! fn domain() -> Domain {
//!     Domain::new().add(
//!         ResourceDef::new("Post")
//!             .uuid_primary_key()
//!             .attribute(Attribute::string("title").required())
//!             .timestamps()
//!             .default_actions(),
//!     )
//! }
//!
//! // In the composition root, after the database extension:
//! app.register(Extension::new(domain)).await;
//!
//! // In a handler or a command:
//! let soot = context.get::<Soot>().await?;
//! let post = soot.create("Post", None, params, &context).await?;
//! ```
//!
//! The domain is handed over as a closure rather than a value, for the same
//! reason [`crate::migrations::SootMigration`] takes one: a migration can be
//! constructed long after the process that registered the domain has moved on,
//! and rebuilding is what keeps that possible and keeps the extension `Send`.

use std::{collections::BTreeMap, sync::Arc};

#[cfg(feature = "migrations")]
use dirtybase_contract::ExtensionMigrations;
use dirtybase_contract::{
    ExtensionSetup, app_contract::Context, cli_contract::CliCommandManager,
    db_contract::base::manager::Manager, http_contract::RouterManager,
    prelude::ContextResourceManager,
};
use dirtybase_db::field_values::FieldValue;

use crate::{
    config::SootConfig,
    context::{ActionContext, Actor, DefaultActionContext},
    data_layer::{DataLayer, RelationalDataLayer},
    domain::Domain,
    error::Result,
    query::Query,
    record::Record,
    resource::ResourceDef,
};

/// A map of attribute name to value, which is what every action takes as input.
pub type Params = BTreeMap<String, FieldValue>;

/// Builds the application's domain.
///
/// `Send + Sync` because the extension is registered globally and read from
/// every request, and `'static` because it outlives the registration.
pub type DomainBuilder = Arc<dyn Fn() -> Domain + Send + Sync>;

/// The dirtybase extension: it puts the domain and its data layer where the
/// rest of the application can reach them.
#[derive(Clone, Default)]
pub struct Extension {
    build: Option<DomainBuilder>,
}

impl Extension {
    /// An extension serving the domain that `build` returns.
    ///
    /// The closure is called once per resolution rather than cached, so a
    /// resource added to the declaration later is picked up without rebuilding
    /// the extension. It is cheap: a declaration is pure data.
    pub fn new<F>(build: F) -> Self
    where
        F: Fn() -> Domain + Send + Sync + 'static,
    {
        Self {
            build: Some(Arc::new(build)),
        }
    }

    /// An extension with an empty domain.
    ///
    /// Useful as a placeholder — in a composition root that registers a
    /// framework extension unconditionally, and only fills in the domain when
    /// the application has one.
    pub fn empty() -> Self {
        Self::default()
    }

    /// The domain this extension serves.
    pub fn build_domain(&self) -> Domain {
        match &self.build {
            Some(build) => build(),
            None => Domain::new(),
        }
    }

    pub async fn config_from_ctx(ctx: &Context) -> std::result::Result<SootConfig, anyhow::Error> {
        ctx.get_config_once::<SootConfig>("soot").await
    }
}

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        let config = match Self::config_from_ctx(context).await {
            Ok(config) => config,
            Err(error) => {
                tracing::warn!(
                    target: "soot",
                    "could not read the soot configuration, using the default: {error}"
                );
                SootConfig::default()
            }
        };

        if !config.is_enabled() {
            tracing::info!(target: "soot", "soot is disabled by configuration");
            return;
        }

        register(self.build_domain(), config).await;
    }

    async fn boot(&mut self, context: &Context) {
        // Extension setup runs here rather than in `setup`, because it may need
        // the `Manager` the database extension registered — and the database
        // extension is registered before this one.
        let Ok(soot) = context.get::<Soot>().await else {
            tracing::trace!(target: "soot", "no domain to set extensions up for");
            return;
        };

        let action_context = ActionContextBridge::from_context(context).await;
        if let Err(errors) = soot.domain().setup_extensions(&action_context).await {
            tracing::warn!(target: "soot", "an extension failed to set up: {errors}");
        }
    }

    async fn shutdown(&mut self, context: &Context) {
        if let Ok(soot) = context.get::<Soot>().await {
            if let Err(errors) = soot.domain().teardown_extensions().await {
                tracing::warn!(target: "soot", "an extension failed to tear down: {errors}");
            }
        }
    }

    fn register_routes(&self, _manager: &mut RouterManager) {
        // No routes. Every resource's actions are named and introspectable, so
        // an API over them is generated from `Domain::describe` by the
        // application that wants one, rather than fixed here.
    }

    async fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        setup_cli(manager)
    }

    #[cfg(feature = "migrations")]
    async fn migrations(&self, _context: &Context) -> Option<ExtensionMigrations> {
        let build = self.build.clone()?;
        Some(vec![Box::new(crate::migrations::SootMigration::new(
            move || build(),
        ))])
    }
}

// ---- the context resource --------------------------------------------------

/// The domain and the data layer over it, together.
///
/// Both halves are needed to run an action, and the pair is what identifies
/// "soot" as a resource: registering the domain and the layer separately would
/// leave a caller free to pair a domain with the wrong layer, which fails at the
/// first relationship rather than at the point of the mistake.
#[derive(Clone)]
pub struct Soot {
    domain: Arc<Domain>,
    data_layer: Arc<dyn DataLayer>,
}

impl std::fmt::Debug for Soot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The data layer is a trait object with no `Debug`, and there is nothing
        // in it worth printing that the resource count does not already say.
        f.debug_struct("Soot")
            .field("resources", &self.domain.resources().len())
            .finish()
    }
}

impl Soot {
    pub fn new(domain: Domain, manager: Manager) -> Self {
        let domain = Arc::new(domain);
        Self {
            data_layer: Arc::new(RelationalDataLayer::with_domain(
                manager,
                Arc::clone(&domain),
            )),
            domain,
        }
    }

    /// A handle over an existing pair, for a caller that built its own data
    /// layer — an in-memory one, or one with a different storage.
    pub fn with_data_layer(domain: Domain, data_layer: Arc<dyn DataLayer>) -> Self {
        Self {
            domain: Arc::new(domain),
            data_layer,
        }
    }

    pub fn domain(&self) -> &Arc<Domain> {
        &self.domain
    }

    pub fn data_layer(&self) -> &Arc<dyn DataLayer> {
        &self.data_layer
    }

    pub fn resource(&self, name: &str) -> Result<&Arc<ResourceDef>> {
        self.domain.resource(name)
    }

    pub fn has_resource(&self, name: &str) -> bool {
        self.domain.has_resource(name)
    }

    /// A summary of the domain, for an API or a CLI to render.
    pub fn describe(&self) -> crate::domain::DomainDescription {
        self.domain.describe()
    }

    /// Create a record, with the caller's dirtybase context as the action
    /// context.
    pub async fn create(
        &self,
        resource: &str,
        action: Option<&str>,
        params: Params,
        context: &Context,
    ) -> Result<Record> {
        let action_context = ActionContextBridge::from_context(context).await;
        self.domain
            .create(&self.data_layer, resource, action, params, &action_context)
            .await
    }

    /// Update the record identified by `id`.
    pub async fn update(
        &self,
        resource: &str,
        action: Option<&str>,
        id: &str,
        params: Params,
        context: &Context,
    ) -> Result<Record> {
        let action_context = ActionContextBridge::from_context(context).await;
        self.domain
            .update(
                &self.data_layer,
                resource,
                action,
                id,
                params,
                &action_context,
            )
            .await
    }

    /// Destroy the record identified by `id`.
    pub async fn destroy(
        &self,
        resource: &str,
        action: Option<&str>,
        id: &str,
        context: &Context,
    ) -> Result<Record> {
        let action_context = ActionContextBridge::from_context(context).await;
        self.domain
            .destroy(&self.data_layer, resource, action, id, &action_context)
            .await
    }

    /// Run a read action.
    pub async fn read(
        &self,
        resource: &str,
        action: Option<&str>,
        configure: impl FnOnce(&mut Query),
        context: &Context,
    ) -> Result<Vec<Record>> {
        let action_context = ActionContextBridge::from_context(context).await;
        self.domain
            .read(
                &self.data_layer,
                resource,
                action,
                configure,
                &action_context,
            )
            .await
    }
}

/// Register the domain and its data layer on the application's context.
///
/// The pair is resolved once and then kept, so `auto_migrate` issues its DDL on
/// the first resolution rather than at registration — the `Manager` it needs is
/// only reachable from a context, and this is the first moment one exists.
pub async fn register(domain: Domain, config: SootConfig) {
    let auto_migrate = config.auto_migrate();

    ContextResourceManager::<Soot>::register(
        move |_| {
            let name = "soot";
            Box::pin(async move { Ok((name, 0).into()) })
        },
        move |context| {
            let domain = domain.clone();
            Box::pin(async move {
                let manager = context
                    .get::<Manager>()
                    .await
                    .map_err(|e| anyhow::anyhow!("soot needs a database manager: {e}"))?;

                if auto_migrate {
                    // Best effort: a deployment that already migrated should not
                    // fail to boot, and the error is worth seeing but not worth
                    // stopping for.
                    if let Err(errors) = crate::schema::create_all_tables(
                        &manager,
                        domain.resources(),
                        Some(&domain),
                    )
                    .await
                    {
                        tracing::warn!(
                            target: "soot",
                            "soot could not create its tables: {errors}"
                        );
                    }
                }

                Ok(Soot::new(domain, manager))
            })
        },
        |_soot| Box::pin(async {}),
    )
    .await;
}

// ---- bridging a dirtybase context into an action context -------------------

/// An [`ActionContext`] backed by a dirtybase [`Context`].
///
/// The two carry the same idea — who is acting, and which tenant they are in —
/// in different shapes, so this is where an authenticated request becomes the
/// [`Actor`] a change or a query filter reads. A dirtybase `Actor` that is
/// absent is not an error: an unauthenticated request is a guest, and soot's
/// own [`Actor`] is exactly that by default.
#[derive(Clone, Default)]
pub struct ActionContextBridge {
    inner: DefaultActionContext,
}

impl ActionContextBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Read the actor and tenant out of a dirtybase context.
    ///
    /// Falls back to a guest context when neither is registered, so this works
    /// in a CLI command and a test as well as in a request.
    pub async fn from_context(context: &Context) -> Self {
        let actor = match context
            .get::<dirtybase_contract::auth_contract::Actor>()
            .await
        {
            Ok(dirtybase_actor) => {
                let mut actor = match actor_id(&dirtybase_actor) {
                    Some(id) => Actor::from_id(id),
                    None => Actor::new(),
                };
                // The username rides along as an attribute so a change can use
                // `actor(:username)` without the caller looking it up.
                actor = actor.put("username", dirtybase_actor.username_ref().to_string());
                actor
            }
            Err(_) => Actor::new(),
        };

        let inner = match context.tenant_context().await {
            Some(tenant) => {
                DefaultActionContext::with_actor_and_tenant(actor, tenant.id_as_string())
            }
            None => DefaultActionContext::with_actor(actor),
        };

        Self { inner }
    }

    /// A bridge over an explicit soot actor and tenant, for a caller running an
    /// action outside a request.
    pub fn with_actor(actor: Actor, tenant: Option<String>) -> Self {
        let inner = match tenant {
            Some(tenant) => DefaultActionContext::with_actor_and_tenant(actor, tenant),
            None => DefaultActionContext::with_actor(actor),
        };
        Self { inner }
    }

    pub fn with_value(mut self, key: &str, value: FieldValue) -> Self {
        self.inner = self.inner.with_value(key, value);
        self
    }
}

/// The actor's id as a soot value, which is what an `actor(:id)` expression or
/// a `created_by` stamp reads.
///
/// This is passed through as the underlying uuid rather than its string form:
/// a `created_by` stamp is a uuid attribute, and coercing the string through
/// soot's uuid type would reject anything that is not a v7 id. Carrying the
/// value straight through keeps the id exactly as the auth layer knows it.
fn actor_id(actor: &dirtybase_contract::auth_contract::Actor) -> Option<FieldValue> {
    // `id` is the auth layer's `ArcUuid7`. Converting to a `FieldValue::Uuid`
    // through dirtybase's `From` keeps the value exactly as the auth layer
    // knows it; coercing the string through soot's uuid type would reject
    // anything that is not a v7 id on the way back out.
    actor.id().map(FieldValue::from)
}

impl ActionContext for ActionContextBridge {
    fn actor(&self) -> Option<&Actor> {
        self.inner.actor()
    }

    fn tenant(&self) -> Option<FieldValue> {
        self.inner.tenant()
    }

    fn get(&self, key: &str) -> Option<FieldValue> {
        self.inner.get(key)
    }

    fn context(&self) -> BTreeMap<String, FieldValue> {
        self.inner.context()
    }

    fn with_value(&self, key: &str, value: FieldValue) -> Box<dyn ActionContext> {
        Box::new(Self {
            inner: self.inner.clone().with_value(key, value),
        })
    }

    fn handle(&self) -> Arc<dyn ActionContext> {
        Arc::new(self.clone())
    }
}

// ---- the `soot` command ----------------------------------------------------

/// Register the `soot` command, which describes the domain from the terminal.
pub fn setup_cli(mut manager: CliCommandManager) -> CliCommandManager {
    let command = dirtybase_contract::cli_contract::clap::Command::new("soot")
        .about("Inspect the soot domain")
        .arg_required_else_help(true)
        .subcommand(
            dirtybase_contract::cli_contract::clap::Command::new("describe")
                .about("Print every resource, its actions and its extensions"),
        )
        .subcommand(
            dirtybase_contract::cli_contract::clap::Command::new("tables")
                .about("Print the resource to table mapping"),
        );

    manager.register(command, |_name, _matches, context| {
        Box::pin(async move {
            let soot = context
                .get::<Soot>()
                .await
                .map_err(|e| anyhow::anyhow!("soot is not registered: {e}"))?;
            let domain = soot.domain();

            println!("-----------------------------------------------------------------");
            println!("                         Soot domain                            ");
            println!("-----------------------------------------------------------------");

            let extensions: Vec<String> = domain
                .all_extensions()
                .iter()
                .map(|extension| extension.name().to_string())
                .collect();
            println!(" extensions: {}", render(&extensions));

            for resource in domain.resources() {
                println!();
                println!(" {} -> {}", resource.name(), resource.table_name());
                println!("   primary key : {}", resource.primary_key_column());
                println!(
                    "   timestamps  : {}",
                    if resource.has_timestamps() {
                        "yes"
                    } else {
                        "no"
                    }
                );
                let applicable: Vec<String> = domain
                    .extensions_of(resource)
                    .iter()
                    .map(|extension| extension.name().to_string())
                    .collect();
                println!("   extensions  : {}", render(&applicable));
                let actions: Vec<String> = resource
                    .actions()
                    .iter()
                    .map(|action| {
                        format!(
                            "{}{}",
                            action.name(),
                            if action.is_primary() {
                                " (primary)"
                            } else {
                                ""
                            }
                        )
                    })
                    .collect();
                println!("   actions     : {}", render(&actions));
            }

            Ok(())
        })
    });

    manager
}

/// `a, b, c`, or `none` for an empty list, so the output is a line either way.
fn render(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
