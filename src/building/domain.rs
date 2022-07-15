use std::error;
use std::fmt;

pub struct Building {
    pub id: i32,
    pub name: String,
    pub color: String,
}

#[derive(Debug)]
pub enum BuildingError {
    BuildingInvalid(String),
    BuildingNotFound(String),
    BuildingNameAlreadyExists(String),
    InternalServerError,
}

#[derive(Debug)]
pub enum BuildingErrorCode {
    BuildingInvalid,
    BuildingNotFound,
    BuildingNameAlreadyExists,
    InternalServerError,
}

impl error::Error for BuildingErrorCode {}

impl fmt::Display for BuildingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildingError::BuildingInvalid(message) => write!(f, "{}", message),
            BuildingError::BuildingNotFound(message) => write!(f, "{}", message),
            BuildingError::BuildingNameAlreadyExists(message) => write!(f, "{}", message),
            BuildingError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

impl fmt::Display for BuildingErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildingErrorCode::BuildingInvalid => write!(f, "BUILDING_INVALID"),
            BuildingErrorCode::BuildingNotFound => write!(f, "BUILDING_NOT_FOUND"),
            BuildingErrorCode::BuildingNameAlreadyExists => write!(f, "BUILDING_NAME_ALREADY_EXISTS"),
            BuildingErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}
