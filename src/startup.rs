use std::net::TcpListener;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Serialize;

use crate::http::AuthenticationMiddlewareFactory;

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
                            .wrap(
                                AuthenticationMiddlewareFactory::new(
                                    auth_service.clone(),
                                    role_service.clone(),
                                )
                                .with_permission("view_list_announcement"),
                            )
                            .route("", web::get().to(auth_http::me)),
                    ),
            )
    })
    .listen(listener)?
    .run();

    println!("Server is starting on http://localhost:8080");

    Ok(server)
}
