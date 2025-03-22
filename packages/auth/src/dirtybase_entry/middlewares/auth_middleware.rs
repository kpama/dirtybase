use std::collections::HashMap;

use dirtybase_contract::http::prelude::*;

pub async fn handle_auth_middleware(
    req: Request,
    next: Next,
    params: Option<HashMap<String, String>>,
) -> impl IntoResponse {
    println!(">>>>>>>>>>>>>>>>>>> In auth middleware: {:#?}", &params);

    next.run(req).await
}
