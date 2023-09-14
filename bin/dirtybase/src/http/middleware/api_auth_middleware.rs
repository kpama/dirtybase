use crate::{
    app::model::{
        dirtybase_user::dirtybase_user_helpers::jwt_manager::JWTManager,
        permission::{permission_service::PermissionService, PermissionValidator},
    },
    http::http_helpers::pluck_jwt_token,
};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use busybody::helpers::provide;
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
#[derive(Debug, Clone)]
pub struct JWTAuth;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for JWTAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct ApiAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ApiAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let jwt = pluck_jwt_token(&req).unwrap_or_default();

        let service = Rc::clone(&self.service);

        // let fut = self.service.call(req);

        Box::pin(async move {
            let jwt_manager = provide::<JWTManager>().await;

            if let Some(result) = jwt_manager.verify_jwt(&jwt) {
                let permission_service: PermissionService = provide().await;
                let permission_validator =
                    PermissionValidator::new(result.into(), permission_service);

                req.extensions_mut().insert(permission_validator);

                Ok(service.call(req).await?)
            } else {
                // TODO: Figure out the right return type
                panic!("*** Authentication failed ***")
            }
        })
    }
}
