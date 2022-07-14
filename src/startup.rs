use std::net::TcpListener;
use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use crate::container::Container;

use crate::auth::http as auth_http;
use crate::role::http as role_http;

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener, container: Container) -> Result<Server, std::io::Error> {
    let container = Arc::new(container);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(container.clone())
            .route("/health_check", web::get().to(health_check))
            .service(
                web::scope("/dashboard")
                    .route("/v1/auth/register", web::post().to(auth_http::register))
                    .route(
                        "/v1/auth/verification/{email}",
                        web::get().to(auth_http::send_email_verification),
                    )
                    .route(
                        "/v1/auth/verification",
                        web::put().to(auth_http::confirm_email_verification),
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
                    .route("/v1/role", web::get().to(role_http::list_role)),
            )
    })
    .listen(listener)?
    .run();

    println!("Server is starting on http://localhost:8080");

    Ok(server)
}
