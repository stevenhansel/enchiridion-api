use std::{
    borrow::Borrow,
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
    features::device::{AuthenticateDeviceError, DeviceErrorCode, DeviceServiceInterface},
    http::HttpErrorResponse,
};

pub type AuthenticationInfoResult = Result<i32, AuthenticateDeviceError>;
pub type AuthenticationInfo = Rc<AuthenticationInfoResult>;

pub struct DeviceAuthenticationMiddleware<S> {
    device_service: Arc<dyn DeviceServiceInterface + Send + Sync + 'static>,
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for DeviceAuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let device_service = self.device_service.clone();

        async move {
            let result = device_service.authenticate(req.headers()).await;
            req.extensions_mut()
                .insert::<AuthenticationInfo>(Rc::new(result));

            Ok(service.call(req).await?)
        }
        .boxed_local()
    }
}

pub struct DeviceAuthenticationMiddlewareFactory {
    device_service: Arc<dyn DeviceServiceInterface + Send + Sync + 'static>,
}

impl DeviceAuthenticationMiddlewareFactory {
    pub fn new(device_service: Arc<dyn DeviceServiceInterface + Send + Sync + 'static>) -> Self {
        DeviceAuthenticationMiddlewareFactory { device_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for DeviceAuthenticationMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = DeviceAuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DeviceAuthenticationMiddleware {
            device_service: self.device_service.clone(),
            service: Rc::new(service),
        }))
    }
}

pub struct DeviceAuthenticationContext(pub Option<AuthenticationInfo>);

impl FromRequest for DeviceAuthenticationContext {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<AuthenticationInfo>().cloned();
        ready(Ok(DeviceAuthenticationContext(value)))
    }
}

impl std::ops::Deref for DeviceAuthenticationContext {
    type Target = Option<AuthenticationInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn get_device_id(auth: DeviceAuthenticationContext) -> Result<i32, AuthenticateDeviceError> {
    if let Some(context) = auth.0 {
        return match context.borrow() {
            Ok(user_id) => Ok(*user_id),
            Err(e) => Err(*e),
        };
    }

    Ok(-1)
}

pub fn parse_device_authentication_middleware_error(e: AuthenticateDeviceError) -> HttpResponse {
    match e {
        AuthenticateDeviceError::AuthenticationFailed(message) => {
            return HttpResponse::Unauthorized().json(HttpErrorResponse::new(
                DeviceErrorCode::AuthenticationFailed.to_string(),
                vec![message.to_string()],
            ))
        }
        AuthenticateDeviceError::DeviceNotFound(message) => {
            return HttpResponse::Unauthorized().json(HttpErrorResponse::new(
                DeviceErrorCode::DeviceNotFound.to_string(),
                vec![message.to_string()],
            ))
        }
        AuthenticateDeviceError::InternalServerError => {
            return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                DeviceErrorCode::InternalServerError.to_string(),
                vec![AuthenticateDeviceError::InternalServerError.to_string()],
            ))
        }
    }
}
