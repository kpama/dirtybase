use dirtybase_contract::{
    auth_contract::Actor,
    prelude::{Observable, observable::cel::CelCoreVariable},
};

pub(crate) async fn register_observers() {
    // Inject the current actor
    CelCoreVariable::subscribe(|mut core, ctx| async move {
        let actor = if let Ok(actor) = ctx.get::<Actor>().await {
            actor
        } else {
            Actor::default()
        };

        core.set_actor(actor);

        core
    })
    .await;
}
