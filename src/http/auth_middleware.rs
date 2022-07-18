// the logic for the auth middleware
// - get the access token jwt from bearer: Authorization
// - decode the token and check whether the token is valid or not / still expired (auth service)
// - return user_id so that it can be accessed in the controller level

use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage,
};
use futures::future::LocalBoxFuture;
use futures::FutureExt;

use crate::auth::AuthServiceInterface;

pub type AuthenticationInfo = Rc<i32>;

pub struct AuthenticationMiddleware<S> {
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let auth_service = self.auth_service.clone();

        async move {
            if let Some(access_token) = req.cookie("access_token") {
                if let Ok(claims) =
                    auth_service.decode_access_token(access_token.value().to_string())
                {
                    if let Ok(user_id) = claims["user_id"].parse::<i32>() {
                        req.extensions_mut()
                            .insert::<AuthenticationInfo>(Rc::new(user_id));
                    }
                }
            }

            Ok(service.call(req).await?)
        }
        .boxed_local()
    }
}

pub struct AuthenticationMiddlewareFactory {
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
}

impl AuthenticationMiddlewareFactory {
    pub fn new(auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>) -> Self {
        AuthenticationMiddlewareFactory { auth_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthenticationMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            auth_service: self.auth_service.clone(),
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticationContext(pub Option<AuthenticationInfo>);

impl FromRequest for AuthenticationContext {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<AuthenticationInfo>().cloned();
        ready(Ok(AuthenticationContext(value)))
    }
}

impl std::ops::Deref for AuthenticationContext {
    type Target = Option<AuthenticationInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
