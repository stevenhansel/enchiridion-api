use std::sync::Arc;

use actix_web::cookie::Cookie;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::http::{
    get_user_id_from_auth_context, ApiValidationError, AuthenticationContext,
    AuthenticationMiddlewareError, AuthenticationMiddlewareErrorCode, HttpErrorResponse,
};

use super::service::AuthServiceInterface;
use super::{AuthError, AuthErrorCode, LoginParams, RegisterParams};

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

pub async fn register(
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
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
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
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
            AuthError::EmailAlreadyConfirmed(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    AuthErrorCode::EmailAlreadyConfirmed.to_string(),
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

#[derive(Debug, Deserialize)]
pub struct VerifyEmailConfirmationTokenQueryParams {
    token: String,
}

pub async fn verify_email_confirmation_token(
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
    query_params: web::Query<VerifyEmailConfirmationTokenQueryParams>,
) -> HttpResponse {
    if let Err(e) = auth_service
        .verify_email_confirmation_token(query_params.token.to_string())
        .await
    {
        match e {
            AuthError::TokenInvalid(message) => {
                return HttpResponse::Unauthorized().json(HttpErrorResponse::new(
                    AuthErrorCode::TokenInvalid.to_string(),
                    vec![message],
                ))
            }
            AuthError::TokenExpired(message) => {
                return HttpResponse::Gone().json(HttpErrorResponse::new(
                    AuthErrorCode::TokenExpired.to_string(),
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

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmEmailBody {
    #[validate(length(min = 1, message = "token: Token must be a valid string"))]
    token: String,
}

pub async fn confirm_email(
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
    body: web::Json<ConfirmEmailBody>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    let result = match auth_service.confirm_email(body.token.to_string()).await {
        Ok(res) => res,
        Err(e) => match e {
            AuthError::TokenInvalid(message) => {
                return HttpResponse::Unauthorized().json(HttpErrorResponse::new(
                    AuthErrorCode::TokenInvalid.to_string(),
                    vec![message],
                ))
            }
            AuthError::TokenExpired(message) => {
                return HttpResponse::Gone().json(HttpErrorResponse::new(
                    AuthErrorCode::TokenExpired.to_string(),
                    vec![message],
                ))
            }
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
        },
    };

    let mut permissions: Vec<PermissionObject> = vec![];
    for p in result.entity.role.permissions {
        permissions.push(PermissionObject {
            id: p.id,
            name: p.name,
        });
    }

    let role = RoleObject {
        id: result.entity.role.id,
        name: result.entity.role.name,
        permissions,
    };

    let response = LoginResponse {
        id: result.entity.id,
        name: result.entity.name,
        email: result.entity.email,
        profile_picture: result.entity.profile_picture,
        is_email_confirmed: result.entity.is_email_confirmed,
        user_status: UserStatusObject {
            value: result.entity.user_status.clone().value().to_string(),
            label: result.entity.user_status.clone().label().to_string(),
        },
        role,
    };

    let access_token_cookie = Cookie::build("access_token", result.access_token)
        .path("/")
        .secure(true)
        .http_only(true)
        .finish();
    let refresh_token_cookie = Cookie::build("refresh_token", result.refresh_token)
        .path("/")
        .secure(true)
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(access_token_cookie)
        .cookie(refresh_token_cookie)
        .json(response)
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginBody {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub is_email_confirmed: bool,
    pub user_status: UserStatusObject,
    pub role: RoleObject,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatusObject {
    label: String,
    value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleObject {
    pub id: i32,
    pub name: String,
    pub permissions: Vec<PermissionObject>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionObject {
    pub id: i32,
    pub name: String,
}

pub async fn login(
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
    body: web::Json<LoginBody>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    let result = match auth_service
        .login(LoginParams {
            email: body.email.to_string(),
            password: body.password.to_string(),
        })
        .await
    {
        Ok(res) => res,
        Err(e) => match e {
            AuthError::AuthenticationFailed(message) => {
                return HttpResponse::Unauthorized().json(HttpErrorResponse::new(
                    AuthErrorCode::AuthenticationFailed.to_string(),
                    vec![message],
                ))
            }
            AuthError::UserNotVerified(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    AuthErrorCode::UserNotVerified.to_string(),
                    vec![message],
                ))
            }
            _ => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AuthErrorCode::InternalServerError.to_string(),
                    vec![AuthError::InternalServerError.to_string()],
                ))
            }
        },
    };

    let mut permissions: Vec<PermissionObject> = vec![];
    for p in result.entity.role.permissions {
        permissions.push(PermissionObject {
            id: p.id,
            name: p.name,
        });
    }

    let role = RoleObject {
        id: result.entity.role.id,
        name: result.entity.role.name,
        permissions,
    };

    let response = LoginResponse {
        id: result.entity.id,
        name: result.entity.name,
        email: result.entity.email,
        profile_picture: result.entity.profile_picture,
        is_email_confirmed: result.entity.is_email_confirmed,
        user_status: UserStatusObject {
            value: result.entity.user_status.clone().value().to_string(),
            label: result.entity.user_status.clone().label().to_string(),
        },
        role,
    };

    let access_token_cookie = Cookie::build("access_token", result.access_token)
        .path("/")
        .secure(true)
        .http_only(true)
        .finish();
    let refresh_token_cookie = Cookie::build("refresh_token", result.refresh_token)
        .path("/")
        .secure(true)
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(access_token_cookie)
        .cookie(refresh_token_cookie)
        .json(response)
}

pub async fn refresh_token(
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
    req: HttpRequest,
) -> HttpResponse {
    let refresh_token = match req.cookie("refresh_token") {
        Some(token) => token,
        _ => {
            return HttpResponse::Unauthorized().json(HttpErrorResponse::new(
                AuthErrorCode::AuthenticationFailed.to_string(),
                vec!["Token not provided".into()],
            ))
        }
    };

    if let Err(e) = auth_service
        .refresh_token(refresh_token.value().to_string())
        .await
    {
        match e {
            AuthError::TokenInvalid(message) => {
                return HttpResponse::Unauthorized().json(HttpErrorResponse::new(
                    AuthErrorCode::TokenInvalid.to_string(),
                    vec![message],
                ))
            }
            AuthError::TokenExpired(message) => {
                return HttpResponse::Gone().json(HttpErrorResponse::new(
                    AuthErrorCode::TokenExpired.to_string(),
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

pub async fn me(auth: AuthenticationContext<'_>) -> HttpResponse {
    let user_id = match get_user_id_from_auth_context(auth) {
        Ok(id) => id,
        Err(e) => match e {
            AuthenticationMiddlewareError::AuthenticationFailed(message) => {
                return HttpResponse::Unauthorized().json(HttpErrorResponse::new(
                    AuthenticationMiddlewareErrorCode::AuthenticationFailed.to_string(),
                    vec![message.to_string()],
                ))
            }
            AuthenticationMiddlewareError::ForbiddenPermission(message) => {
                return HttpResponse::Forbidden().json(HttpErrorResponse::new(
                    AuthenticationMiddlewareErrorCode::ForbiddenPermission.to_string(),
                    vec![message.to_string()],
                ))
            }
            AuthenticationMiddlewareError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AuthenticationMiddlewareErrorCode::InternalServerError.to_string(),
                    vec![AuthenticationMiddlewareError::InternalServerError.to_string()],
                ))
            }
        },
    };

    println!("user_id: {}", user_id);

    HttpResponse::Ok().finish()
}

pub async fn forgot_password() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn reset_password() -> HttpResponse {
    HttpResponse::NoContent().finish()
}
