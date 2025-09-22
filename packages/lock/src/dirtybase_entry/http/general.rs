use dirtybase_contract::http_contract::prelude::*;

pub(crate) async fn index_handler() -> impl IntoResponse {
    Html("Be awesome")
}
