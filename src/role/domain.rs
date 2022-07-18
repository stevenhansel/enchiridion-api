use std::error;
use std::fmt;

pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

pub struct Permission {
    pub id: i32,
    pub name: String,
    pub label: String,
}

#[derive(Debug)]
pub enum RoleError {
    InternalServerError(String),
}

impl error::Error for RoleError {}

impl fmt::Display for RoleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RoleError::InternalServerError(message) => write!(f, "{}", message),
        }
    }
}

#[derive(Debug)]
pub enum GetPermissionByUserIdError {
    InternalServerError(String),
}

impl error::Error for GetPermissionByUserIdError {}

impl fmt::Display for GetPermissionByUserIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GetPermissionByUserIdError::InternalServerError(message) => write!(f, "{}", message),
        }
    }
}
