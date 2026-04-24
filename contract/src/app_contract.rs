mod cancellation_token;
mod context;

pub mod observable;
use busybody::helpers::set_type;
pub use cancellation_token::*;
pub use context::*;

pub async fn global_context() -> Context {
    if let Some(ctx) = busybody::helpers::get_type::<Context>().await {
        ctx
    } else {
        let ctx = Context::make_global().await;
        set_type(ctx.clone()).await;
        ctx
    }
}

pub async fn make_context() -> Context {
    _ = global_context().await;
    Context::new().await
}
