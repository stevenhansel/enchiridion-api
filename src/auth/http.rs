use actix_web::{web::Json, HttpResponse};
use serde::{Deserialize, Serialize};
use shaku_actix::Inject;

use crate::auth::service::RegisterParams;
use crate::container::Container;

use super::service::AuthServiceInterface;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterBody {
    name: String,
    email: String,
    password: String,
    reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    id: i32,
    name: String,
    email: String,
    registration_reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    message: String,
}

pub async fn register(
    body: Json<RegisterBody>,
    auth_service: Inject<Container, dyn AuthServiceInterface>,
) -> HttpResponse {
    let params = RegisterParams {
        name: body.name.to_string(),
        email: body.email.to_string(),
        password: body.password.to_string(),
        reason: body.reason.to_string(),
    };
    match auth_service.register(params).await {
        Ok(user) => user,
        Err(e) => {
            println!("{}", e.to_string());
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: e.to_string(),
            });
        }
    };

    HttpResponse::NoContent().finish()
}

pub async fn send_email_verification() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn confirm_email_verification() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn forgot_password() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn reset_password() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn login() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn refresh_token() -> HttpResponse {
    HttpResponse::NoContent().finish()
}
