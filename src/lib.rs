use std::net::TcpListener;
use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::web::Json;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use shaku_actix::Inject;
use sqlx::{Pool, Postgres};

mod container;
use container::container::Container;

mod auth;
mod user;

use auth::service::{AuthServiceInterface, RegisterParams};
use user::repository::{UserRepository, UserRepositoryParameters};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterBody {
    name: String,
    email: String,
    password: String,
    reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterResponse {
    id: i32,
    name: String,
    email: String,
    registration_reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    message: String,
}

async fn register(
    body: Json<RegisterBody>,
    auth_service: Inject<Container, dyn AuthServiceInterface>,
) -> impl Responder {
    let params = RegisterParams {
        name: body.name.clone(),
        email: body.email.clone(),
        password: body.password.clone(),
        reason: body.reason.clone(),
    };
    let user = match auth_service.register(params).await {
        Ok(user) => user,
        Err(e) => {
            println!("{}", e.to_string());
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: e.to_string(),
            });
        }
    };

    let response = RegisterResponse {
        id: user.id,
        name: user.name,
        email: user.email,
        registration_reason: user.registration_reason,
    };

    HttpResponse::Created().json(response)
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
            .route("/v1/register", web::post().to(register))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
