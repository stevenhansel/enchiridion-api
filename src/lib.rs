use std::net::TcpListener;
use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use sqlx::{Pool, Postgres};

mod container;
use container::{Container, UserRepository, UserRepositoryParameters};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

async fn register() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener, pool: Pool<Postgres>) -> Result<Server, std::io::Error> {
    let container = Arc::new(
        Container::builder()
            .with_component_parameters::<UserRepository>(UserRepositoryParameters { _db: pool })
            .build(),
    );

    let server = HttpServer::new(move || {
        App::new()
            .app_data(container.clone())
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
