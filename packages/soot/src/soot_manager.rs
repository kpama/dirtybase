use std::{collections::HashMap, fmt::Debug, sync::Arc};

use dirtybase_contract::app_contract::Context;
use dirtybase_db::base::manager::Manager;

use crate::{
    dirtybase_entry::{ActionContextBridge, Soot},
    domain::Domain,
};

#[derive(Clone)]
pub struct SootManager {
    domains: Arc<HashMap<String, Domain>>,
    context: Context,
}

impl Debug for SootManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dg = f.debug_struct("SootManager");
        dg.field("total domains", &self.domains.len());
        for (name, domain) in self.domains.iter() {
            let name = format!("domain::{}", name);
            dg.field(&name, domain);
        }

        dg.finish()
    }
}

impl SootManager {
    pub(crate) fn new(domains: Vec<Domain>, context: Context) -> Self {
        Self {
            context,
            domains: domains
                .into_iter()
                .fold(HashMap::new(), |mut acc, entry| {
                    acc.insert(entry.name().to_string(), entry);
                    acc
                })
                .into(),
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.domains.contains_key(name)
    }

    pub async fn soot(&self, name: &str) -> Option<Soot> {
        let manager = match self.context.get::<Manager>().await {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("could not instantiate soot. {}", e);
                return None;
            }
        };

        if let Some(domain) = self.domains.get(name).cloned() {
            Some(Soot::new(domain, manager, self.context.clone()).await)
        } else {
            None
        }
    }

    pub(crate) async fn create_all_tables(&self, manager: Manager) {
        // Best effort: a deployment that already migrated should not
        // fail to boot, and the error is worth seeing but not worth
        // stopping for.
        for domain in self.domains.values() {
            if let Err(errors) =
                crate::schema::create_all_tables(&manager, domain.resources(), Some(&domain)).await
            {
                tracing::warn!(
                    target: "soot",
                    "soot could not create its tables: {errors}"
                );
            }
        }
    }

    pub(crate) async fn setup_extensions(&self) {
        let action_context = ActionContextBridge::from_context(&self.context).await;
        for domain in self.domains.values() {
            if let Err(errors) = domain.setup_extensions(&action_context).await {
                tracing::warn!(target: "soot", "an extension failed to set up: {errors}");
            }
        }
    }

    pub(crate) async fn teardown_extensions(&self) {
        for domain in self.domains.values() {
            if let Err(errors) = domain.teardown_extensions().await {
                tracing::warn!(target: "soot", "an extension failed to tear down: {errors}");
            }
        }
    }
}
