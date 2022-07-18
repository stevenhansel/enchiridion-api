// the logic for the auth middleware
// - get the access token jwt from bearer: Authorization
// - decode the token and check whether the token is valid or not / still expired (auth service)
// - return user_id so that it can be accessed in the controller level
use core::fmt;
use std::{
    error,
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

use crate::{
    auth::{AuthError, AuthServiceInterface},
    role::RoleServiceInterface,
};

#[derive(Debug, Clone, Copy)]
pub enum AuthenticationMiddlewareError<'a> {
    AuthenticationFailed(&'a str),
    ForbiddenPermission(&'a str),
    InternalServerError,
}

#[derive(Debug)]
pub enum AuthenticationMiddlewareErrorCode {
    AuthenticationFailed,
    ForbiddenPermission,
    InternalServerError,
}

impl<'a> error::Error for AuthenticationMiddlewareError<'a> {}

impl<'a> fmt::Display for AuthenticationMiddlewareError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthenticationMiddlewareError::AuthenticationFailed(message) => {
                write!(f, "{}", message)
            }
            AuthenticationMiddlewareError::ForbiddenPermission(message) => write!(f, "{}", message),
            AuthenticationMiddlewareError::InternalServerError => {
                write!(f, "Internal Server Error")
            }
        }
    }
}

impl fmt::Display for AuthenticationMiddlewareErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthenticationMiddlewareErrorCode::AuthenticationFailed => {
                write!(f, "AUTHENTICATION_FAILED")
            }
            AuthenticationMiddlewareErrorCode::ForbiddenPermission => {
                write!(f, "FORBIDDEN_PERMISSION")
            }
            AuthenticationMiddlewareErrorCode::InternalServerError => {
                write!(f, "INTERNAL_SERVER_ERROR")
            }
        }
    }
}

pub type AuthenticationInfoResult<'a> = Result<i32, AuthenticationMiddlewareError<'a>>;
pub type AuthenticationInfo<'a> = Rc<AuthenticationInfoResult<'a>>;

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
        // let role_service = self.role_service.clone();
        // let permission = self.permission.clone();

        async move {
            // TOOD: move the func to the service
            let func = || {
                let access_token = match req.cookie("access_token") {
                    Some(cookie) => cookie.value().to_string(),
                    None => {
                        return Err(AuthenticationMiddlewareError::AuthenticationFailed(
                            "Authentication failed, Token expired or invalid",
                        ))
                    }
                };

                let claims = match auth_service.decode_access_token(access_token) {
                    Ok(claims) => claims,
                    Err(e) => match e {
                        AuthError::TokenInvalid(_) => {
                            return Err(AuthenticationMiddlewareError::AuthenticationFailed(
                                "Authentication failed, Token expired or invalid",
                            ))
                        }
                        AuthError::TokenExpired(_) => {
                            return Err(AuthenticationMiddlewareError::AuthenticationFailed(
                                "Authentication failed, Token expired or invalid",
                            ))
                        }
                        _ => return Err(AuthenticationMiddlewareError::InternalServerError),
                    },
                };

                let user_id = match claims["user_id"].parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => {
                        return Err(AuthenticationMiddlewareError::AuthenticationFailed(
                            "Authentication failed, Token expired or invalid",
                        ))
                    }
                };

                Ok(user_id)
            };

                // if let Some(permission) = permission {
                //     let role_id = match claims["role_id"].parse::<i32>() {
                //         Ok(id) => id,
                //         Err(_) => {
                //             return Err(AuthenticationMiddlewareError::AuthenticationFailed(
                //                 "Authentication failed, Token expired or invalid",
                //             ))
                //         }
                //     };

                //     let permissions: Vec<String> = match role_service
                //         .get_permissions_by_role_id(role_id).await
                //     {
                //         Ok(permissions) => permissions.into_iter().map(|p| p.name).collect(),
                //         Err(_) => return Err(AuthenticationMiddlewareError::InternalServerError),
                //     };

                //     if !permissions.contains(&permission) {
                //         return Err(AuthenticationMiddlewareError::ForbiddenPermission(
                //             "User doesn't have the permission to access the designated route",
                //         ));
                //     }
                // }


            let result = func();
            req.extensions_mut()
                .insert::<AuthenticationInfo>(Rc::new(result));

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

pub struct AuthenticationContext<'a>(pub Option<AuthenticationInfo<'a>>);

impl FromRequest for AuthenticationContext<'_> {
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

impl<'a> std::ops::Deref for AuthenticationContext<'a> {
    type Target = Option<AuthenticationInfo<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn get_user_id_from_auth_context(
    auth: AuthenticationContext,
) -> Result<i32, AuthenticationMiddlewareError> {
    if let Some(context) = auth.0 {
        return match *context {
            Ok(user_id) => Ok(user_id),
            Err(e) => Err(e),
        };
    } else {
        return Err(AuthenticationMiddlewareError::InternalServerError);
    }
}
