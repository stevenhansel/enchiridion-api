use std::future::{ready, Ready};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::dev::{forward_ready, Server, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, App, Error, FromRequest, HttpMessage, HttpResponse, HttpServer};
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use serde::Serialize;

use crate::auth::{http as auth_http, AuthServiceInterface};
use crate::building::{http as building_http, BuildingServiceInterface};
use crate::role::{http as role_http, RoleServiceInterface};
use crate::user::UserServiceInterface;

#[derive(Serialize)]
struct HealthCheckResponse {
    status: String,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthCheckResponse {
        status: "healthy".into(),
    })
}

pub fn run(
    listener: TcpListener,
    role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
    building_service: Arc<dyn BuildingServiceInterface + Send + Sync + 'static>,
    user_service: Arc<dyn UserServiceInterface + Send + Sync + 'static>,
    auth_service: Arc<dyn AuthServiceInterface + Send + Sync + 'static>,
) -> Result<Server, std::io::Error> {
    let role_svc = web::Data::new(role_service.clone());
    let building_svc = web::Data::new(building_service.clone());
    let user_svc = web::Data::new(user_service.clone());
    let auth_svc = web::Data::new(auth_service.clone());

    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(role_svc.clone())
            .app_data(building_svc.clone())
            .app_data(user_svc.clone())
            .app_data(auth_svc.clone())
            .route("/", web::get().to(health_check))
            .service(
                web::scope("/dashboard")
                    .route("/v1/auth/register", web::post().to(auth_http::register))
                    .route(
                        "/v1/auth/verification/{email}",
                        web::get().to(auth_http::send_email_confirmation),
                    )
                    .route(
                        "/v1/auth/verification",
                        web::get().to(auth_http::verify_email_confirmation_token),
                    )
                    .route(
                        "/v1/auth/verification",
                        web::put().to(auth_http::confirm_email),
                    )
                    .route("/v1/auth/login", web::post().to(auth_http::login))
                    .route("/v1/auth/refresh", web::put().to(auth_http::refresh_token))
                    .route("/v1/roles", web::get().to(role_http::list_role))
                    .route(
                        "/v1/auth/forgot-password",
                        web::get().to(auth_http::forgot_password),
                    )
                    .route(
                        "/v1/auth/reset-password",
                        web::put().to(auth_http::reset_password),
                    )
                    .route(
                        "/v1/buildings",
                        web::get().to(building_http::list_buildings),
                    )
                    .route("/v1/buildings", web::post().to(building_http::create))
                    .route(
                        "/v1/buildings/{buildingId}",
                        web::put().to(building_http::update),
                    )
                    .route(
                        "/v1/buildings/{buildingId}",
                        web::delete().to(building_http::delete),
                    )
                    .service(
                        web::scope("/v1/me")
                            .wrap(AuthenticationMiddlewareFactory::new(auth_service.clone()))
                            .route("", web::get().to(auth_http::me)),
                    )
            )
    })
    .listen(listener)?
    .run();

    println!("Server is starting on http://localhost:8080");

    Ok(server)
}

// the logic for the auth middleware
// - get the access token jwt from bearer: Authorization
// - decode the token and check whether the token is valid or not / still expired (auth service)
// - return user_id so that it can be accessed in the controller level

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
            println!("a");
            if let Some(access_token) = req.cookie("access_token") {
                println!("b");
                if let Ok(claims) =
                    auth_service.decode_access_token(access_token.value().to_string())
                {
                        println!("c");
                    if let Ok(user_id) = claims["user_id"].parse::<i32>() {
                            println!("d");
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
