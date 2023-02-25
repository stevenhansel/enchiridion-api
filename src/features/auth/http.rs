use std::sync::Arc;

use actix_web::cookie::{Cookie, SameSite};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    features::role::RoleServiceInterface,
    http::{
        derive_authentication_middleware_error, derive_user_id, ApiValidationError,
        AuthenticationContext, HttpErrorResponse, API_VALIDATION_ERROR_CODE,
    },
};

use super::service::AuthServiceInterface;
use super::{AuthError, AuthErrorCode, ChangePasswordError, LoginParams, RegisterParams};

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
    role: String,
    building_id: i32,
}

pub async fn register(
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
    role_service: web::Data<Arc<dyn RoleServiceInterface + Send + Sync + 'static>>,
    body: web::Json<RegisterBody>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    let role_values: Vec<&'static str> = role_service
        .list_role()
        .into_iter()
        .map(|r| r.value)
        .collect();

    if !role_values.contains(&body.role.as_str()) {
        return HttpResponse::BadRequest().json(HttpErrorResponse::new(
            API_VALIDATION_ERROR_CODE.to_string(),
            vec!["Value of the role is invalid".into()],
        ));
    }

    if let Err(e) = auth_service
        .register(RegisterParams {
            name: body.name.to_string(),
            email: body.email.to_string(),
            password: body.password.to_string(),
            reason: body.reason.clone(),
            role: body.role.clone(),
            building_id: body.building_id,
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

    let mut building: Option<BuildingObject> = None;
    if let Some(b) = result.entity.building {
        building = Some(BuildingObject {
            id: b.id,
            name: b.name,
            color: b.color,
            created_at: b.created_at,
            updated_at: b.updated_at,
        })
    }

    let response = AuthEntityObject {
        id: result.entity.id,
        name: result.entity.name,
        email: result.entity.email,
        profile_picture: result.entity.profile_picture,
        is_email_confirmed: result.entity.is_email_confirmed,
        user_status: UserStatusObject {
            value: result.entity.user_status.clone().value().to_string(),
            label: result.entity.user_status.clone().label().to_string(),
        },
        role: RoleObject {
            name: result.entity.role.name,
            description: result.entity.role.description,
            permissions: result
                .entity
                .role
                .permissions
                .into_iter()
                .map(|p| PermissionObject {
                    label: p.label,
                    value: p.value,
                })
                .collect(),
        },
        building,
    };

    let access_token_cookie = Cookie::build("access_token", result.access_token)
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(SameSite::None)
        .finish();
    let refresh_token_cookie = Cookie::build("refresh_token", result.refresh_token)
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(SameSite::None)
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
pub struct AuthEntityObject {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub is_email_confirmed: bool,
    pub user_status: UserStatusObject,
    pub role: RoleObject,
    pub building: Option<BuildingObject>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatusObject {
    label: String,
    value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildingObject {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleObject {
    pub name: &'static str,
    pub description: &'static str,
    pub permissions: Vec<PermissionObject>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionObject {
    pub label: &'static str,
    pub value: &'static str,
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

    let mut building: Option<BuildingObject> = None;
    if let Some(b) = result.entity.building {
        building = Some(BuildingObject {
            id: b.id,
            name: b.name,
            color: b.color,
            created_at: b.created_at,
            updated_at: b.updated_at,
        })
    }

    let response = AuthEntityObject {
        id: result.entity.id,
        name: result.entity.name,
        email: result.entity.email,
        profile_picture: result.entity.profile_picture,
        is_email_confirmed: result.entity.is_email_confirmed,
        user_status: UserStatusObject {
            value: result.entity.user_status.clone().value().to_string(),
            label: result.entity.user_status.clone().label().to_string(),
        },
        role: RoleObject {
            name: result.entity.role.name,
            description: result.entity.role.description,
            permissions: result
                .entity
                .role
                .permissions
                .into_iter()
                .map(|p| PermissionObject {
                    label: p.label,
                    value: p.value,
                })
                .collect(),
        },
        building,
    };

    let access_token_cookie = Cookie::build("access_token", result.access_token)
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(SameSite::None)
        .finish();
    let refresh_token_cookie = Cookie::build("refresh_token", result.refresh_token)
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(SameSite::None)
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

    let result = match auth_service
        .refresh_token(refresh_token.value().to_string())
        .await
    {
        Ok(result) => result,
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

    let access_token_cookie = Cookie::build("access_token", result.access_token)
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(SameSite::None)
        .finish();
    let refresh_token_cookie = Cookie::build("refresh_token", result.refresh_token)
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(SameSite::None)
        .finish();

    HttpResponse::NoContent()
        .cookie(access_token_cookie)
        .cookie(refresh_token_cookie)
        .finish()
}

pub async fn me(
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
) -> HttpResponse {
    let user_id = match derive_user_id(auth) {
        Ok(id) => id,
        Err(e) => return derive_authentication_middleware_error(e),
    };

    let entity = match auth_service.me(user_id).await {
        Ok(entity) => entity,
        Err(e) => match e {
            AuthError::UserNotFound(e) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AuthErrorCode::UserNotFound.to_string(),
                    vec![e.to_string()],
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

    let mut building: Option<BuildingObject> = None;
    if let Some(b) = entity.building {
        building = Some(BuildingObject {
            id: b.id,
            name: b.name,
            color: b.color,
            created_at: b.created_at,
            updated_at: b.updated_at,
        })
    }

    let response = AuthEntityObject {
        id: entity.id,
        name: entity.name,
        email: entity.email,
        profile_picture: entity.profile_picture,
        is_email_confirmed: entity.is_email_confirmed,
        user_status: UserStatusObject {
            value: entity.user_status.clone().value().to_string(),
            label: entity.user_status.clone().label().to_string(),
        },
        role: RoleObject {
            name: entity.role.name,
            description: entity.role.description,
            permissions: entity
                .role
                .permissions
                .into_iter()
                .map(|p| PermissionObject {
                    label: p.label,
                    value: p.value,
                })
                .collect(),
        },
        building,
    };

    HttpResponse::Ok().json(response)
}

pub async fn logout(
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
) -> HttpResponse {
    let user_id = match derive_user_id(auth) {
        Ok(id) => id,
        Err(e) => return derive_authentication_middleware_error(e),
    };

    match auth_service.logout(user_id).await {
        Ok(entity) => entity,
        Err(e) => match e {
            AuthError::UserNotFound(e) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AuthErrorCode::UserNotFound.to_string(),
                    vec![e.to_string()],
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

    let access_token_cookie = Cookie::build("access_token", "")
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(SameSite::None)
        .finish();
    let refresh_token_cookie = Cookie::build("refresh_token", "")
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(SameSite::None)
        .finish();

    HttpResponse::NoContent()
        .cookie(access_token_cookie)
        .cookie(refresh_token_cookie)
        .finish()
}

pub async fn forgot_password() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn reset_password() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordBody {
    #[validate(length(
        min = 8,
        message = "password: Password must have at least 8 characters"
    ))]
    old_password: String,
    #[validate(length(
        min = 8,
        message = "password: Password must have at least 8 characters"
    ))]
    new_password: String,
}

pub async fn change_password(
    auth_service: web::Data<Arc<dyn AuthServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    body: web::Json<ChangePasswordBody>,
) -> HttpResponse {
    let user_id = match derive_user_id(auth) {
        Ok(id) => id,
        Err(e) => return derive_authentication_middleware_error(e),
    };

    if let Err(e) = body.validate() {
        let e = ApiValidationError::new(e);

        return HttpResponse::BadRequest().json(HttpErrorResponse::new(e.code(), e.messages()));
    }

    if let Err(e) = auth_service
        .change_password(
            user_id,
            body.old_password.clone(),
            body.new_password.clone(),
        )
        .await
    {
        match e {
            ChangePasswordError::UserInvalidOldPassword(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AuthErrorCode::UserInvalidOldPassword.to_string(),
                    vec![message.into()],
                ))
            }
            ChangePasswordError::UserNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    AuthErrorCode::UserNotFound.to_string(),
                    vec![message.into()],
                ))
            }
            ChangePasswordError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    AuthErrorCode::InternalServerError.to_string(),
                    vec![ChangePasswordError::InternalServerError.to_string()],
                ))
            }
        }
    }

    HttpResponse::NoContent().finish()
}
