mod context;

pub use context::*;

pub async fn global_context() -> Context {
    if let Some(ctx) = busybody::helpers::get_type::<Context>().await {
        ctx
    } else {
        Context::make_global().await
    }
}
