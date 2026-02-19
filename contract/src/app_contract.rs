mod cancellation_token;
mod context;

pub use cancellation_token::*;
pub use context::*;
pub use dirtybase_common::app::*;

pub async fn global_context() -> Context {
    if let Some(ctx) = busybody::helpers::get_type::<Context>().await {
        ctx
    } else {
        Context::make_global().await
    }
}
