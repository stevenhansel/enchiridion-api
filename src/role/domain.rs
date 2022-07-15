use std::error;
use std::fmt;

pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
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
