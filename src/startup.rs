use std::net::TcpListener;
use std::rc::Rc;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::dev::{forward_ready, Server, Service, ServiceRequest, ServiceResponse};
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use futures::FutureExt;
use futures::future::LocalBoxFuture;
use serde::Serialize;

use crate::auth::{http as auth_http, AuthServiceInterface};
use crate::building::{http as building_http, BuildingServiceInterface};
use crate::role::{http as role_http, RoleServiceInterface};
use crate::user::{UserRepositoryInterface, UserServiceInterface};

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
    let role_service = web::Data::new(role_service);
    let building_service = web::Data::new(building_service);
    let user_service = web::Data::new(user_service);
    let auth_service = web::Data::new(auth_service);

    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(role_service.clone())
            .app_data(building_service.clone())
            .app_data(user_service.clone())
            .app_data(auth_service.clone())
            .wrap(cors)
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
                    .route("/v1/roles", web::get().to(role_http::list_role)),
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

pub trait AuthenticationMiddlewareInterface {}

pub struct AuthenticationMiddleware<S> {
    auth_service: Arc<dyn UserRepositoryInterface + Send + Sync + 'static>,
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
            // let access_token = match req.cookie("access_token") {
            //     Some(token) => token,
            //     None => 
            // };
            Ok(service.call(req).await?)
        }.boxed_local()
    }
}
