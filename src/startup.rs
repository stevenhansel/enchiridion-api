use std::net::TcpListener;
use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use crate::container::Container;

use crate::auth::http as auth_http;

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
                web::scope("/dashboard").service(
                    web::scope("/v1/auth")
                        .route("/register", web::post().to(auth_http::register))
                        .route(
                            "/verification/{email}",
                            web::get().to(auth_http::send_email_verification),
                        )
                        .route(
                            "/verification",
                            web::put().to(auth_http::confirm_email_verification),
                        )
                        .route("/login", web::post().to(auth_http::login))
                        .route("/refresh", web::put().to(auth_http::refresh_token))
                        .route(
                            "/forgot-password",
                            web::get().to(auth_http::forgot_password),
                        )
                        .route("/reset-password", web::put().to(auth_http::reset_password)),
                ),
            )
    })
    .listen(listener)?
    .run();

    println!("Server is starting on http://localhost:8080");

    Ok(server)
}
