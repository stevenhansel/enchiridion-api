use core::fmt;

use crate::{building::Building, device::Device};

#[derive(Debug)]
pub struct Floor {
    pub id: i32,
    pub name: String,
    pub building: Building,
    pub devices: Option<Vec<Device>>,
}

pub enum FloorErrorCode {
    FloorAlreadyExists,
    BuildingNotFound,
    InternalServerError,
}

impl std::fmt::Display for FloorErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FloorErrorCode::FloorAlreadyExists => write!(f, "FLOOR_ALREADY_EXISTS"),
            FloorErrorCode::BuildingNotFound => write!(f, "BUILDING_NOT_FOUND"),
            FloorErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}

pub enum CreateFloorError {
    FloorAlreadyExists(String),
    BuildingNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for CreateFloorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CreateFloorError::FloorAlreadyExists(message) => write!(f, "{}", message),
            CreateFloorError::BuildingNotFound(message) => write!(f, "{}", message),
            CreateFloorError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}
