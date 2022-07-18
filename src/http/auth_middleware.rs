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
    cookie::Cookie,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage,
};
use futures::future::LocalBoxFuture;
use futures::FutureExt;

use crate::{auth::AuthServiceInterface, role::RoleServiceInterface};

pub enum AuthenticationMiddlewareError {
    AuthenticationFailed(String),
    ForbiddenPermission(String),
    InternalServerError,
}

pub type AuthenticationInfoResult = Result<i32, AuthenticationMiddlewareError>;
pub type AuthenticationInfo = Rc<AuthenticationInfoResult>;

pub struct AuthenticationMiddleware<S> {
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
    role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
    permission: Option<String>,
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
            let result: Result<String, AuthenticationMiddlewareError> =
                match req.cookie("access_token") {
                    Some(cookie) => Ok(cookie.value().to_string()),
                    None => Err(AuthenticationMiddlewareError::AuthenticationFailed(
                        "Authentication Failed, Token expired or invalid".to_string(),
                    )),
                };
            if let Ok(access_token) = result {
                // let result = auth_service.decode_access_token(access_token);
            } else if let Err(e) = result {
                req.extensions_mut()
                    .insert::<AuthenticationInfoResult>(Err(e));
            }

            Ok(service.call(req).await?)
        }
        .boxed_local()
    }
}

pub struct AuthenticationMiddlewareFactory {
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
    role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
    permission: Option<String>,
}

impl AuthenticationMiddlewareFactory {
    pub fn new(
        auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
        role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
    ) -> Self {
        AuthenticationMiddlewareFactory {
            auth_service,
            role_service,
            permission: None,
        }
    }

    pub fn with_permission(mut self, permission: &str) -> Self {
        self.permission = Some(permission.to_string());
        self
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
            role_service: self.role_service.clone(),
            permission: self.permission.clone(),
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
