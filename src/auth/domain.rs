use std::error;
use std::fmt;

use crate::user::UserStatus;

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

#[derive(Debug)]
pub enum AuthErrorCode {
    AuthenticationFailed,
    TokenInvalid,
    TokenExpired,
    EmailAlreadyExists,
    EmailAlreadyConfirmed,
    RoleNotFound,
    InternalServerError,
    UserNotFound,
    UserNotVerified,
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

pub struct UserAuthEntity {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub is_email_confirmed: bool,
    pub user_status: UserStatus,
    pub role: RoleAuthEntity,
}

pub struct RoleAuthEntity {
    pub id: i32,
    pub name: String,
    pub permissions: Vec<PermissionAuthEntity>,
}

pub struct PermissionAuthEntity {
    pub id: i32,
    pub name: String,
}

pub struct RefreshTokenResult {
    pub access_token: String,
    pub refresh_token: String,
}
