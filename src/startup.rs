use std::net::TcpListener;
use std::sync::Arc;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::dev::Server;

use crate::container::Container;

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener, container: Container) -> Result<Server, std::io::Error> {
    let container = Arc::new(container);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(container.clone())
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    println!("Server is starting on http://localhost:8080");

    Ok(server)
}
