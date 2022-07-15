use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use shaku_actix::Inject;
use validator::Validate;

use crate::auth::service::RegisterParams;
use crate::container::Container;
use crate::http::{ApiValidationError, HttpErrorResponse};

use super::service::AuthServiceInterface;
use super::{AuthError, AuthErrorCode};

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterBody {
    #[validate(length(min = 1, message = "name: Name must not be empty"))]
    name: String,
    #[validate(email(message = "email: Must be a valid email address"))]
    email: String,
    #[validate(length(
        min = 8,
        message = "password: Password must have at least 8 characters"
    ))]
    password: String,
    reason: Option<String>,
    role_id: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    message: String,
}

pub async fn register(
    auth_service: Inject<Container, dyn AuthServiceInterface>,
    body: web::Json<RegisterBody>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    if let Err(e) = auth_service
        .register(RegisterParams {
            name: body.name.to_string(),
            email: body.email.to_string(),
            password: body.password.to_string(),
            reason: body.reason.clone(),
            role_id: body.role_id,
        })
        .await
    {
        match e {
            AuthError::EmailAlreadyExists(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    AuthErrorCode::EmailAlreadyExists.to_string(),
                    vec![message],
                ))
            }
            AuthError::RoleNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AuthErrorCode::RoleNotFound.to_string(),
                    vec![message],
                ))
            }
            _ => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AuthErrorCode::InternalServerError.to_string(),
                    vec![AuthError::InternalServerError.to_string()],
                ))
            }
        }
    }

    HttpResponse::NoContent().finish()
}

pub async fn send_email_confirmation(
    auth_service: Inject<Container, dyn AuthServiceInterface>,
    path: web::Path<String>,
) -> HttpResponse {
    let email = path.into_inner();

    if let Err(e) = auth_service.send_email_confirmation(email).await {
        match e {
            AuthError::UserNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AuthErrorCode::UserNotFound.to_string(),
                    vec![message],
                ))
            }
            _ => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AuthErrorCode::InternalServerError.to_string(),
                    vec![AuthError::InternalServerError.to_string()],
                ))
            }
        }
    }

    HttpResponse::NoContent().finish()
}

pub async fn confirm_email() -> HttpResponse {
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
