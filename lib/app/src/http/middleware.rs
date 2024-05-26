pub async fn my_middleware_test(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let company_id = "company unknown";
    let app_id = "app unknown";

    log::info!("company: {:?}, app: {}", company_id, app_id);

    // axum::response::Response::new("Hello world".into())
    next.run(request).await
}

pub async fn api_auth_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let jwt = request.headers().get("authorization");

    log::info!("auth: {:?}", jwt);

    next.run(request).await
}
