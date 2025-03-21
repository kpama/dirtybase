use std::collections::HashMap;

use dirtybase_contract::http::prelude::*;

pub async fn handle_auth_middleware(
    req: Request,
    next: Next,
    _params: Option<HashMap<String, String>>,
) -> impl IntoResponse {
    println!(">>>>>>>>>>>>>>>>>>> In auth middleware");

    //next.run(req).await
    "Response from auth middleware"
}
