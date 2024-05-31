use dirtybase_contract::http::RouterManager;

pub mod general;

pub fn register(mut manager: RouterManager) -> RouterManager {
    manager = general::register(manager);

    manager
}
