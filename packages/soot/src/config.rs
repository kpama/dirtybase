//! Configuration for the dirtybase extension.
//!
//! The engine itself takes no configuration: a domain is data and running it
//! needs nothing but a `Manager`. What is configurable here is only the
//! framework's behaviour around it — whether the extension participates at all,
//! and whether its tables are created as part of boot.
//!
//! Keys live under `DTY_SOOT`, or in a `soot.toml` picked up through
//! `optional_file`:
//!
//! ```toml
//! [soot]
//! enable = true
//! # Create the domain's tables on boot rather than only through `migrate up`.
//! auto_migrate = false
//! # Skip the `tenant_id` stamping an extension would otherwise add.
//! stamp_tenant = true
//! ```

use async_trait::async_trait;
use dirtybase_contract::{
    app_contract::Context,
    config_contract::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
    prelude::config::Config,
};
use serde::Deserialize;

/// The `[soot]` table.
///
/// `Deserialize` is what a per-tenant JSON override is decoded into, and
/// `#[serde(default)]` is what lets that JSON be partial — a tenant overriding
/// only `auto_migrate` still gets the defaults for the rest, which is what
/// `SootConfig::default` documents.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SootConfig {
    enable: bool,
    auto_migrate: bool,
    stamp_tenant: Option<String>,
}

impl Default for SootConfig {
    fn default() -> Self {
        // On by default: a caller who registered the extension asked for it, and
        // an extension that does nothing unless a flag is flipped is a worse
        // default than one that works.
        Self {
            enable: true,
            // Off by default. Migrations are a deployment decision, and quietly
            // issuing DDL on boot is exactly the thing that surprises people.
            auto_migrate: false,
            stamp_tenant: None,
        }
    }
}

impl SootConfig {
    pub fn is_enabled(&self) -> bool {
        self.enable
    }

    pub fn set_enable(&mut self, enable: bool) -> &mut Self {
        self.enable = enable;
        self
    }

    pub fn auto_migrate(&self) -> bool {
        self.auto_migrate
    }

    pub fn set_auto_migrate(&mut self, auto_migrate: bool) -> &mut Self {
        self.auto_migrate = auto_migrate;
        self
    }

    /// The attribute the tenant filter extension should scope to, if the
    /// application wants tenant scoping at all.
    pub fn stamp_tenant(&self) -> Option<&str> {
        self.stamp_tenant.as_deref()
    }

    pub fn set_stamp_tenant(&mut self, attribute: Option<&str>) -> &mut Self {
        self.stamp_tenant = attribute.map(str::to_string);
        self
    }
}

impl From<Config> for SootConfig {
    fn from(config: Config) -> Self {
        let mut built = SootConfig::default();
        // `ok()` rather than `expect`: a partially filled file is the normal
        // case, and a missing key is not a malformed one.
        if let Some(enable) = config.get_bool("enable").ok() {
            built.enable = enable;
        }
        if let Some(auto_migrate) = config.get_bool("auto_migrate").ok() {
            built.auto_migrate = auto_migrate;
        }
        if let Some(attribute) = config.get_string("stamp_tenant").ok() {
            built.stamp_tenant = Some(attribute);
        }
        built
    }
}

#[async_trait]
impl TryFromDirtyConfig for SootConfig {
    type Returns = Self;

    async fn from_config(config: &DirtyConfig, _ctx: &Context) -> ConfigResult<Self::Returns> {
        let built = config
            .optional_file("soot.toml", Some("DTY_SOOT"))
            .build()
            .await
            .expect("could not create soot configuration");
        Ok(Self::from(built))
    }
}
