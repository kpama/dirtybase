use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, RwLock},
};

use crate::prelude::{Context, observable::cel::CommonExpressionSandbox};

#[derive(Debug, Default, Clone)]
pub struct PermissionManager {
    wildcards: Arc<RwLock<HashMap<String, Wildcard>>>,
}

impl PermissionManager {
    const PARTS_SEPARATOR: char = ':';
    const SUB_PARTS_SEPARATOR: char = ',';
    const ALL: &str = "*";

    /// Create a new instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if the actor has all the permission specified
    /// All permissions must return true
    pub async fn all(&self, actions: &[&str], context: &Context) -> bool {
        for an_action in actions {
            if !self.can(*an_action, context).await {
                return false;
            }
        }

        true
    }

    /// Checks if the actor has at least one of the permissions
    pub async fn any(&self, actions: &[&str], context: &Context) -> bool {
        for an_action in actions {
            if self.can(*an_action, context).await {
                return true;
            }
        }

        false
    }

    /// Checks if an actor has the specified permission
    pub async fn can<T: ToString>(&self, action: T, context: &Context) -> bool {
        let temp = action.to_string();
        let action = temp.trim();
        let mut is_well_formed = !action.is_empty();

        let mut pieces = action
            .split(Self::PARTS_SEPARATOR)
            .map(|entry| entry.trim())
            .map(String::from)
            .map(|part| {
                if part.is_empty() {
                    is_well_formed = false;
                }
                part
            })
            .collect::<Vec<String>>()
            .into_iter();

        if !is_well_formed {
            return false;
        }

        let mut ans = false;
        if let Ok(r_lock) = self.wildcards.read() {
            if let Some(all) = r_lock.get(Self::ALL) {
                if all.children.is_some() {
                    let mut temp = pieces.clone();
                    _ = temp.next();

                    if all.implies(temp) {
                        ans = true;
                    }
                } else {
                    ans = true;
                }
            }
        }

        tracing::trace!("checking ability: {}, result: {}", &action, ans);
        if ans && self.apply_condition(context, action).await {
            return true;
        }

        if let Some(first) = pieces.next() {
            let first = first.trim();
            if first.is_empty() {
                return false;
            }

            if let Ok(r_lock) = self.wildcards.read() {
                if let Some(wild) = r_lock.get(first) {
                    ans = wild.implies(pieces);
                }
            }
        }

        tracing::trace!("checking children abilities: {}, result: {}", &action, ans);
        if !ans {
            return ans;
        }

        ans && self.apply_condition(context, action).await
    }

    async fn apply_condition(&self, context: &Context, name: &str) -> bool {
        match context.get::<CommonExpressionSandbox>().await {
            Ok(sandbox) => match sandbox.execute(context, name).await {
                Ok(result) => return result.is_zero() == false,
                Err(crate::prelude::observable::cel::CelManagerError::ProgramNotFound) => {
                    return true;
                }
                Err(crate::prelude::observable::cel::CelManagerError::ExecutionError(e)) => {
                    tracing::error!("error executing CEL program: {}, {}", name, e);
                }
            },
            Err(e) => {
                tracing::error!("could not get CEL sandbox: {}", e);
            }
        }

        false
    }

    fn walk<T: Iterator<Item = String>>(wildcard: &mut Wildcard, mut parts: T) {
        if let Some(x) = parts.next() {
            if wildcard.children.is_none() {
                wildcard.children = Some(HashMap::new());
            }

            let p = parts.collect::<Vec<String>>();
            x.split(Self::SUB_PARTS_SEPARATOR)
                .map(String::from)
                .for_each(|e| {
                    let mut child = Wildcard::new();
                    Self::walk(&mut child, p.clone().into_iter());
                    wildcard.children.as_mut().unwrap().insert(e, child);
                });
        } else {
            if wildcard.children.is_none() {
                wildcard.children = Some(HashMap::new());
            }

            wildcard
                .children
                .as_mut()
                .unwrap()
                .insert("*".to_string(), Wildcard::new());
        }
    }

    pub fn add<T: ToString>(&mut self, entry: T) -> &mut Self {
        self.add_str(&entry.to_string())
    }

    pub fn add_str(&mut self, name: &str) -> &mut Self {
        let mut parts = name
            .split(Self::PARTS_SEPARATOR)
            .map(String::from)
            .collect::<VecDeque<String>>();

        if let Some(front) = parts.pop_front() {
            let front_pieces = front
                .split(Self::SUB_PARTS_SEPARATOR)
                .map(String::from)
                .collect::<Vec<String>>();

            for p in front_pieces {
                if let Ok(mut w_lock) = self.wildcards.write() {
                    if let Some(wild) = w_lock.get_mut(&p) {
                        Self::walk(wild, parts.clone().into_iter());
                    } else {
                        let mut wild = Wildcard::new();
                        Self::walk(&mut wild, parts.clone().into_iter());
                        w_lock.insert(p, wild);
                    }
                }
            }
        }

        self
    }
}

impl<T: Iterator<Item = I>, I: ToString> From<T> for PermissionManager {
    fn from(collection: T) -> Self {
        let mut instance = Self::default();

        collection.for_each(|entry| {
            instance.add(entry);
        });

        instance
    }
}

#[derive(Debug, Default)]
struct Wildcard {
    children: Option<HashMap<String, Wildcard>>,
}

impl Wildcard {
    pub fn new() -> Self {
        Self { children: None }
    }

    pub fn implies<I: Iterator<Item = String>>(&self, mut pieces: I) -> bool {
        if let Some(collection) = &self.children {
            if collection.contains_key(PermissionManager::ALL) {
                if let Some(all) = collection.get(PermissionManager::ALL)
                    && all.children.is_some()
                {
                    _ = pieces.next();

                    return all.implies(pieces);
                }

                _ = pieces.next();
                return true;
            }

            if let Some(first) = pieces.next() {
                if let Some(child) = collection.get(&first) {
                    if child.children.is_none() {
                        return pieces.next().is_none();
                    } else {
                        return child.implies(pieces);
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod test {

    use crate::{
        auth_contract::Permission,
        prelude::{
            Observable,
            observable::cel::{CelContext, CommonExpressionSandbox},
        },
    };

    use super::*;

    #[tokio::test]
    async fn test_from_permission_collection() {
        let perms = [
            Permission::new("posts:create", "Can create posts"),
            Permission::new("products:create", "Can create products"),
        ];

        let service = PermissionManager::from(perms.iter());
        let context = Context::new().await;

        // truth cases
        assert!(service.can("posts:create", &context).await);
        assert!(service.can("products:create", &context).await);

        // false cases
        assert!(!service.can("posts:view", &context).await);
        assert!(!service.can("products:edit", &context).await);
    }

    #[tokio::test]
    async fn wrongly_formed_permissions() {
        let mut service = PermissionManager::new();
        let context = Context::new().await;
        service.add_str("posts");
        service.add_str("products");

        assert!(!service.can(":::", &context).await);
        assert!(!service.can("", &context).await);
        assert!(!service.can("        ", &context).await);
        assert!(!service.can(",", &context).await);
        assert!(!service.can(",,,", &context).await);
        assert!(!service.can("posts:", &context).await);
        assert!(!service.can("posts::123", &context).await);
        assert!(!service.can("posts:create:", &context).await);

        assert!(service.can("posts", &context).await);
        assert!(service.can("posts:*", &context).await);
        assert!(service.can("products:*", &context).await);
    }

    #[tokio::test]
    async fn test_one_part() {
        let context = Context::new().await;
        let mut service = PermissionManager::new();
        service.add_str("posts");
        service.add_str("products");

        // truth cases
        assert!(service.can("posts:create", &context).await);
        assert!(service.can("posts:delete", &context).await);
        assert!(service.can("posts:some-other-action", &context).await);
        assert!(service.can("products:view", &context).await);
        assert!(
            service
                .all(&["products:view", "posts:create"], &context)
                .await
        );
        assert!(service.all(&["products", "posts"], &context).await);
        assert!(service.any(&["products", "posts"], &context).await);
        assert!(
            service
                .any(&["products:create", "resource_x:delete"], &context)
                .await
        );

        // false cases
        assert!(!service.can("images:create", &context).await);
        assert!(!service.can("images:delete", &context).await);
        assert!(!service.can("images:some-other-action", &context).await);
        assert!(!service.can("customers:view", &context).await);
        assert!(
            !service
                .all(&["products:view", "images:create"], &context)
                .await
        );
        assert!(
            !service
                .all(&["products", "posts", "images"], &context)
                .await
        );
        assert!(!service.any(&["images", "resource_x"], &context).await);
        assert!(
            !service
                .any(&["images:create", "resource_x:delete"], &context)
                .await
        );
    }

    #[tokio::test]
    async fn test_two_subparts() {
        let context = Context::new().await;
        let mut service = PermissionManager::new();
        service.add_str("posts:create");
        service.add_str("products:create");
        service.add_str("create:orders");

        // truth cases
        assert!(service.can("posts:create", &context).await);
        assert!(service.can("products:create", &context).await);
        assert!(service.can("create:orders", &context).await);

        // false cases
        assert!(!service.can("resource_x:create", &context).await);
        assert!(!service.can("create:resource_x", &context).await);
        assert!(!service.can("posts:view", &context).await);
        assert!(!service.can("products:delete", &context).await);
        assert!(!service.can("edit:orders", &context).await);
    }

    #[tokio::test]
    async fn test_three_parts() {
        let context = Context::new().await;
        let mut service = PermissionManager::new();
        service.add_str("posts:create:123");
        service.add_str("products:create:123");
        service.add_str("create:orders:*");

        // truth cases
        assert!(service.can("posts:create:123", &context).await);
        assert!(service.can("products:create:123", &context).await);
        assert!(service.can("create:orders", &context).await);
        assert!(service.can("create:orders:542", &context).await);

        // false cases
        assert!(!service.can("posts:create", &context).await);
        assert!(!service.can("posts:create:*", &context).await);
        assert!(!service.can("products:create", &context).await);
        assert!(!service.can("products:delete", &context).await);
        assert!(!service.can("products:delete:543", &context).await);
        assert!(!service.can("products:delete:123", &context).await);
    }

    #[tokio::test]
    async fn test_wildcard() {
        let context = Context::new().await;
        let mut service = PermissionManager::new();
        service.add_str("*");

        assert!(service.can("products:delete", &context).await);
    }

    #[tokio::test]
    async fn test_permission_condition() {
        let context = Context::new().await;
        let mut service = PermissionManager::new();
        let cel_sandbox = CommonExpressionSandbox::new().await;

        // permission
        service.add_str("products:delete");

        // add programs
        cel_sandbox.add_program("products:delete", "owner_id == actor_id");
        context.set(cel_sandbox).await;

        CelContext::subscribe(|mut cel, _| async move {
            cel.add_variable("owner_id", 10).unwrap();
            cel.add_variable("actor_id", 10).unwrap();
            cel
        })
        .await;

        assert!(service.can("products:delete", &context).await);
    }
}
