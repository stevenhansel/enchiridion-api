use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage, HttpResponse,
};
use futures::future::LocalBoxFuture;
use futures::FutureExt;

use crate::{
    auth::{AuthErrorCode, AuthServiceInterface, AuthenticateError},
    http::HttpErrorResponse,
};

pub type AuthenticationInfoResult = Result<i32, AuthenticateError>;
pub type AuthenticationInfo = Rc<AuthenticationInfoResult>;

pub struct AuthenticationMiddleware<S> {
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
    permission: Option<&'static str>,
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
        let permission = self.permission.clone();

        async move {
            let result = auth_service
                .authenticate(req.cookie("access_token"), permission)
                .await;
            req.extensions_mut()
                .insert::<AuthenticationInfo>(Rc::new(result));

            Ok(service.call(req).await?)
        }
        .boxed_local()
    }
}

pub struct AuthenticationMiddlewareFactory {
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
    permission: Option<&'static str>,
}

impl AuthenticationMiddlewareFactory {
    pub fn new(auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>) -> Self {
        AuthenticationMiddlewareFactory {
            auth_service,
            permission: None,
        }
    }

    pub fn with_permission(mut self, permission: &'static str) -> Self {
        self.permission = Some(permission);
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
            permission: self.permission,
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

pub fn derive_user_id(auth: AuthenticationContext) -> Result<i32, AuthenticateError> {
    if let Some(context) = auth.0 {
        let ctx = Rc::try_unwrap(context).unwrap();

        return match ctx {
            Ok(user_id) => Ok(user_id),
            Err(e) => Err(e),
        };
    }

    // TODO: if middleware doesn't exist
    Ok(-1)
}

pub fn derive_authentication_middleware_error(e: AuthenticateError) -> HttpResponse {
    match e {
        AuthenticateError::AuthenticationFailed(message) => {
            return HttpResponse::Unauthorized().json(HttpErrorResponse::new(
                AuthErrorCode::AuthenticationFailed.to_string(),
                vec![message.to_string()],
            ))
        }
        AuthenticateError::ForbiddenPermission(message) => {
            return HttpResponse::Forbidden().json(HttpErrorResponse::new(
                AuthErrorCode::ForbiddenPermission.to_string(),
                vec![message.to_string()],
            ))
        }
        AuthenticateError::InternalServerError => {
            return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                AuthErrorCode::InternalServerError.to_string(),
                vec![AuthenticateError::InternalServerError.to_string()],
            ))
        }
    }
}
