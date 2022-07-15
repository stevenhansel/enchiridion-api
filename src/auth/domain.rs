use std::error;
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    EmailAlreadyExists(String),
    RoleNotFound(String),
    InternalServerError(String),
}

#[derive(Debug)]
pub enum AuthErrorCode {
    EmailAlreadyExists,
    RoleNotFound,
    InternalServerError,
}

impl error::Error for AuthError {}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::EmailAlreadyExists(message) => write!(f, "{}", message),
            AuthError::RoleNotFound(message) => write!(f, "{}", message),
            AuthError::InternalServerError(message) => write!(f, "{}", message),
        }
    }
}

impl fmt::Display for AuthErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthErrorCode::EmailAlreadyExists => write!(f, "EMAIL_ALREADY_EXISTS"),
            AuthErrorCode::RoleNotFound => write!(f, "ROLE_NOT_FOUND"),
            AuthErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}
