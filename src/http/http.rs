use std::net::TcpListener;
use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use sqlx::{Pool, Postgres};

use crate::container::container::Container;

use crate::auth::http as auth_http;
use crate::user::repository::{UserRepository, UserRepositoryParameters};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn serve(listener: TcpListener, pool: Pool<Postgres>) -> Result<Server, std::io::Error> {
    let container = Arc::new(
        Container::builder()
            .with_component_parameters::<UserRepository>(UserRepositoryParameters { _db: pool })
            .build(),
    );

    let server = HttpServer::new(move || {
        App::new()
            .app_data(container.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/v1/register", web::post().to(auth_http::register))
    })
    .listen(listener)?
    .run();

    println!("Server is starting on http://localhost:8080");

    Ok(server)
}
