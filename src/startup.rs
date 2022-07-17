use std::future::{ready, Ready};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::dev::{forward_ready, Server, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Serialize;

use crate::container::Container;

use crate::auth::http as auth_http;
use crate::building::http as building_http;
use crate::role::{http as role_http, RoleServiceInterface};
use crate::user::UserRepositoryInterface;

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
    container: Container,
    role_service: Arc<dyn RoleServiceInterface + Send + Sync + 'static>,
) -> Result<Server, std::io::Error> {
    let container = Arc::new(container);
    let role_service = web::Data::new(role_service);

    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(container.clone())
            .app_data(role_service.clone())
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

// the logic for the auth middleware
// - get the access token jwt from bearer: Authorization
// - decode the token and check whether the token is valid or not / still expired (auth service)
// - return user_id so that it can be accessed in the controller level

// pub type AuthenticationInfo = Rc<i32>;

// pub trait AuthenticationMiddlewareInterface {}

// pub struct AuthenticationMiddleware<S> {
//     auth_service: Arc<dyn UserRepositoryInterface>,
//     service: Rc<S>,
// }
