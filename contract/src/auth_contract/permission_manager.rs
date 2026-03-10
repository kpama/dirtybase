use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, RwLock},
};

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
    pub fn all(&self, actions: &[&str]) -> bool {
        for an_action in actions {
            if !self.can(*an_action) {
                return false;
            }
        }

        true
    }

    /// Checks if the actor has at least one of the permissions
    pub fn any(&self, actions: &[&str]) -> bool {
        for an_action in actions {
            if self.can(*an_action) {
                return true;
            }
        }

        false
    }

    /// Checks if an actor has the specified permission
    pub fn can<T: ToString>(&self, action: T) -> bool {
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

        if let Ok(r_lock) = self.wildcards.read() {
            if let Some(all) = r_lock.get(Self::ALL) {
                if all.children.is_some() {
                    let mut temp = pieces.clone();
                    _ = temp.next();

                    if all.implies(temp) {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }

        if let Some(first) = pieces.next() {
            let first = first.trim();
            if first.is_empty() {
                return false;
            }

            if let Ok(r_lock) = self.wildcards.read() {
                if let Some(wild) = r_lock.get(first) {
                    return wild.implies(pieces);
                }
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

    use crate::auth_contract::Permission;

    use super::*;

    #[test]
    fn test_from_permission_collection() {
        let perms = [
            Permission::new("posts:create", "Can create posts"),
            Permission::new("products:create", "Can create products"),
        ];

        let service = PermissionManager::from(perms.iter());

        // truth cases
        assert!(service.can("posts:create"));
        assert!(service.can("products:create"));

        // false cases
        assert!(!service.can("posts:view"));
        assert!(!service.can("products:edit"));
    }

    #[test]
    fn wrongly_formed_permissions() {
        let mut service = PermissionManager::new();
        service.add_str("posts");
        service.add_str("products");

        assert!(!service.can(":::"));
        assert!(!service.can(""));
        assert!(!service.can("        "));
        assert!(!service.can(","));
        assert!(!service.can(",,,"));
        assert!(!service.can("posts:"));
        assert!(!service.can("posts::123"));
        assert!(!service.can("posts:create:"));

        assert!(service.can("posts"));
        assert!(service.can("posts:*"));
        assert!(service.can("products:*"));
    }

    #[test]
    fn test_one_part() {
        let mut service = PermissionManager::new();
        service.add_str("posts");
        service.add_str("products");

        // truth cases
        assert!(service.can("posts:create"));
        assert!(service.can("posts:delete"));
        assert!(service.can("posts:some-other-action"));
        assert!(service.can("products:view"));
        assert!(service.all(&["products:view", "posts:create"]));
        assert!(service.all(&["products", "posts"]));
        assert!(service.any(&["products", "posts"]));
        assert!(service.any(&["products:create", "resource_x:delete"]));

        // false cases
        assert!(!service.can("images:create"));
        assert!(!service.can("images:delete"));
        assert!(!service.can("images:some-other-action"));
        assert!(!service.can("customers:view"));
        assert!(!service.all(&["products:view", "images:create"]));
        assert!(!service.all(&["products", "posts", "images"]));
        assert!(!service.any(&["images", "resource_x"]));
        assert!(!service.any(&["images:create", "resource_x:delete"]));
    }

    #[test]
    fn test_two_subparts() {
        let mut service = PermissionManager::new();
        service.add_str("posts:create");
        service.add_str("products:create");
        service.add_str("create:orders");

        // truth cases
        assert!(service.can("posts:create"));
        assert!(service.can("products:create"));
        assert!(service.can("create:orders"));

        // false cases
        assert!(!service.can("resource_x:create"));
        assert!(!service.can("create:resource_x"));
        assert!(!service.can("posts:view"));
        assert!(!service.can("products:delete"));
        assert!(!service.can("edit:orders"));
    }

    #[test]
    fn test_three_parts() {
        let mut service = PermissionManager::new();
        service.add_str("posts:create:123");
        service.add_str("products:create:123");
        service.add_str("create:orders:*");

        // truth cases
        assert!(service.can("posts:create:123"));
        assert!(service.can("products:create:123"));
        assert!(service.can("create:orders"));
        assert!(service.can("create:orders:542"));

        // false cases
        assert!(!service.can("posts:create"));
        assert!(!service.can("posts:create:*"));
        assert!(!service.can("products:create"));
        assert!(!service.can("products:delete"));
        assert!(!service.can("products:delete:543"));
        assert!(!service.can("products:delete:123"));
    }
}
