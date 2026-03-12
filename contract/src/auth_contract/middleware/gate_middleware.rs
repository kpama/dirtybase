use axum::{extract::Request, middleware::Next, response::IntoResponse};

use crate::{
    auth_contract::{Gate, GateResponse},
    prelude::{Context, MiddlewareParam},
};

const LOG_TARGET: &str = "gate-middleware";

/// Auth middleware that automatically call the gate
pub async fn handle_gate_middleware(
    req: Request,
    param: MiddlewareParam,
    next: Next,
) -> impl IntoResponse {
    tracing::trace!(target = LOG_TARGET, "kind : {}", param.kind_ref());

    if param.kind_ref().is_empty() {
        return next.run(req).await;
    }

    let Some(context) = req.extensions().get::<Context>().cloned() else {
        tracing::error!(target = LOG_TARGET, "could not get context instance");
        return GateResponse::deny().into();
    };

    let Ok(gate) = context.get::<Gate>().await else {
        tracing::error!(target = LOG_TARGET, "could not get gate instance");
        return GateResponse::deny().into();
    };

    let response = gate.response(param.kind_ref()).await;
    if response == false {
        return response.into();
    }

    next.run(req).await
}
