use std::future::{Ready, ready};
use std::net::TcpListener;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::dev::{forward_ready, Server, Transform, ServiceRequest, ServiceResponse, Service};
use actix_web::{web, App, HttpResponse, HttpServer, Error};
use serde::Serialize;
use futures::future::LocalBoxFuture;

use crate::container::Container;

use crate::auth::http as auth_http;
use crate::building::http as building_http;
use crate::role::http as role_http;

#[derive(Serialize)]
struct HealthCheckResponse {
    status: String,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthCheckResponse {
        status: "healthy".into(),
    })
}

pub fn run(listener: TcpListener, container: Container) -> Result<Server, std::io::Error> {
    let container = Arc::new(container);

    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(container.clone())
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
                    .route("/v1/buildings", web::put().to(building_http::update))
                    .route("/v1/buildings", web::delete().to(building_http::delete))
                    .route("/v1/roles", web::get().to(role_http::list_role)),
            )
    })
    .listen(listener)?
    .run();

    println!("Server is starting on http://localhost:8080");

    Ok(server)
}

pub struct Authentication;

pub struct SayHi;

impl<S, B> Transform<S, ServiceRequest> for SayHi
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SayHiMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SayHiMiddleware { service }))
    }
}

pub struct SayHiMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SayHiMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            println!("Hi from response");
            Ok(res)
        })
    }
}
