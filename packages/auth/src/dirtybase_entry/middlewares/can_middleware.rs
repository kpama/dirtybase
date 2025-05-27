use dirtybase_contract::{
    auth_contract::Gate,
    prelude::{IntoResponse, MiddlewareParam, Next, Request},
};

pub async fn handle_can_middleware(
    req: Request,
    param: MiddlewareParam,
    next: Next,
) -> impl IntoResponse {
    println!(">>>>> can middleware <<<<<<");
    println!("params: {:#?}", &param);
    println!("kind: {}", param.kind_ref());

    let response = Gate::new().response(param.kind_ref()).await;
    if response == false {
        return response.into();
    }

    next.run(req).await
}
