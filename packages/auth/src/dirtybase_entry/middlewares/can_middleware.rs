use dirtybase_contract::{
    auth_contract::{Gate, GateResponse},
    prelude::{Context, IntoResponse, MiddlewareParam, Next, Request},
};

pub async fn handle_can_middleware(
    req: Request,
    param: MiddlewareParam,
    next: Next,
) -> impl IntoResponse {
    println!(">>>>> can middleware <<<<<<");
    println!("params: {:#?}", &param);
    println!("kind: {}", param.kind_ref());

    if param.kind_ref().is_empty() {
        return next.run(req).await;
    }

    let Some(context) = req.extensions().get::<Context>().cloned() else {
        tracing::error!(target = "can_middleware", "could not get context instace");
        return GateResponse::deny().into();
    };

    let Ok(gate) = context.get::<Gate>().await else {
        tracing::error!(target = "can_middleware", "could not get gate instace");
        return GateResponse::deny().into();
    };

    let response = gate.response(param.kind_ref()).await;
    if response == false {
        return response.into();
    }

    next.run(req).await
}
