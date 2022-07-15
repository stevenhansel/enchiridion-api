use std::error;
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    TokenInvalid(String),
    TokenExpired(String),
    EmailAlreadyExists(String),
    EmailAlreadyConfirmed(String),
    RoleNotFound(String),
    UserNotFound(String),
    InternalServerError,
}

#[derive(Debug)]
pub enum AuthErrorCode {
    TokenInvalid,
    TokenExpired,
    EmailAlreadyExists,
    EmailAlreadyConfirmed,
    RoleNotFound,
    InternalServerError,
    UserNotFound,
}

impl error::Error for AuthError {}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::TokenInvalid(message) => write!(f, "{}", message),
            AuthError::TokenExpired(message) => write!(f, "{}", message),
            AuthError::EmailAlreadyExists(message) => write!(f, "{}", message),
            AuthError::EmailAlreadyConfirmed(message) => write!(f, "{}", message),
            AuthError::RoleNotFound(message) => write!(f, "{}", message),
            AuthError::UserNotFound(message) => write!(f, "{}", message),
            AuthError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

impl fmt::Display for AuthErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthErrorCode::TokenInvalid => write!(f, "TOKEN_INVALID"),
            AuthErrorCode::TokenExpired => write!(f, "TOKEN_EXPIRED"),
            AuthErrorCode::EmailAlreadyExists => write!(f, "EMAIL_ALREADY_EXISTS"),
            AuthErrorCode::EmailAlreadyConfirmed => write!(f, "EMAIL_ALREADY_CONFIRMED"),
            AuthErrorCode::RoleNotFound => write!(f, "ROLE_NOT_FOUND"),
            AuthErrorCode::UserNotFound => write!(f, "USER_NOT_FOUND"),
            AuthErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}