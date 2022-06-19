use actix_web::{web::Json, HttpResponse};
use serde::{Deserialize, Serialize};
use shaku_actix::Inject;

use super::service::AuthServiceInterface;
use crate::{auth::service::RegisterParams, container::container::Container};

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
