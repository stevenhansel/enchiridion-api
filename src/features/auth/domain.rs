use std::error;
use std::fmt;

use crate::features::{role::RoleObject, user::UserStatus};

#[derive(Debug)]
pub enum AuthError {
    AuthenticationFailed(String),
    TokenInvalid(String),
    TokenExpired(String),
    EmailAlreadyExists(String),
    EmailAlreadyConfirmed(String),
    RoleNotFound(String),
    UserNotFound(String),
    UserNotVerified(String),
    InternalServerError,
}

impl error::Error for AuthError {}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::AuthenticationFailed(message) => write!(f, "{}", message),
            AuthError::TokenInvalid(message) => write!(f, "{}", message),
            AuthError::TokenExpired(message) => write!(f, "{}", message),
            AuthError::EmailAlreadyExists(message) => write!(f, "{}", message),
            AuthError::EmailAlreadyConfirmed(message) => write!(f, "{}", message),
            AuthError::RoleNotFound(message) => write!(f, "{}", message),
            AuthError::UserNotFound(message) => write!(f, "{}", message),
            AuthError::UserNotVerified(message) => write!(f, "{}", message),
            AuthError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

#[derive(Debug)]
pub enum AuthErrorCode {
    AuthenticationFailed,
    TokenInvalid,
    TokenExpired,
    EmailAlreadyExists,
    EmailAlreadyConfirmed,
    RoleNotFound,
    UserNotFound,
    UserNotVerified,
    UserInvalidOldPassword,
    ForbiddenPermission,
    InternalServerError,
}

impl fmt::Display for AuthErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthErrorCode::AuthenticationFailed => write!(f, "AUTHENTICATION_FAILED"),
            AuthErrorCode::TokenInvalid => write!(f, "TOKEN_INVALID"),
            AuthErrorCode::TokenExpired => write!(f, "TOKEN_EXPIRED"),
            AuthErrorCode::EmailAlreadyExists => write!(f, "EMAIL_ALREADY_EXISTS"),
            AuthErrorCode::EmailAlreadyConfirmed => write!(f, "EMAIL_ALREADY_CONFIRMED"),
            AuthErrorCode::RoleNotFound => write!(f, "ROLE_NOT_FOUND"),
            AuthErrorCode::UserNotFound => write!(f, "USER_NOT_FOUND"),
            AuthErrorCode::UserNotVerified => write!(f, "USER_NOT_VERIFIED"),
            AuthErrorCode::UserInvalidOldPassword => write!(f, "USER_INVALID_OLD_PASSWORD"),
            AuthErrorCode::ForbiddenPermission => write!(f, "FORBIDDEN_PERMISSION"),
            AuthErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}

pub struct AuthEntity {
    pub entity: UserAuthEntity,
    pub access_token: String,
    pub refresh_token: String,
}

pub struct UserStatusObject {
    pub label: String,
    pub value: UserStatus,
}

#[derive(Debug)]
pub struct UserAuthEntity {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub is_email_confirmed: bool,
    pub user_status: UserStatus,
    pub role: RoleObject,
    pub building: Option<BuildingAuthEntity>,
}

#[derive(Debug)]
pub struct BuildingAuthEntity {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct RefreshTokenResult {
    pub access_token: String,
    pub refresh_token: String,
}

pub enum ChangePasswordError {
    UserNotFound(&'static str),
    UserInvalidOldPassword(&'static str),
    InternalServerError,
}

impl std::fmt::Display for ChangePasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangePasswordError::UserNotFound(message) => write!(f, "{}", message),
            ChangePasswordError::UserInvalidOldPassword(message) => write!(f, "{}", message),
            ChangePasswordError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum SeedDefaultUserError {
    EmailAlreadyExists(&'static str),
    InternalServerError,
}

impl std::fmt::Display for SeedDefaultUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeedDefaultUserError::EmailAlreadyExists(message) => write!(f, "{}", message),
            SeedDefaultUserError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AuthenticateError {
    AuthenticationFailed(&'static str),
    ForbiddenPermission(&'static str),
    InternalServerError,
}

impl std::fmt::Display for AuthenticateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthenticateError::AuthenticationFailed(message) => write!(f, "{}", message),
            AuthenticateError::ForbiddenPermission(message) => write!(f, "{}", message),
            AuthenticateError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}
