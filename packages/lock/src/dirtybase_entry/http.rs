mod general;
use dirtybase_contract::prelude::RouterManager;

pub(crate) fn register_routes(manager: &mut RouterManager) {
  // prefix all routes with the crate's name
  let prefix = format!("/{}", env!("CARGO_PKG_NAME"));

  // general routes 
  manager.general(Some(&prefix), |router| {
    router.get_x("/", general::index_handler);
  });
  
}