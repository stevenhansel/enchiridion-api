use core::fmt;

#[derive(Debug)]
pub struct Floor {
    pub id: i32,
    pub name: String,
    pub building: BuildingFloorContent,
    pub devices: Vec<DeviceFloorContent>,
}

#[derive(Debug, Clone)]
pub struct BuildingFloorContent {
    pub id: i32,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct DeviceFloorContent {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub total_announcements: i32,
}

pub enum FloorErrorCode {
    FloorNotFound,
    FloorAlreadyExists,
    DeviceCascadeConstraint,
    BuildingNotFound,
    InternalServerError,
}

impl std::fmt::Display for FloorErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FloorErrorCode::FloorNotFound => write!(f, "FLOOR_NOT_FOUND"),
            FloorErrorCode::FloorAlreadyExists => write!(f, "FLOOR_ALREADY_EXISTS"),
            FloorErrorCode::DeviceCascadeConstraint => write!(f, "DEVICE_CASCADE_CONSTRAINT"),
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

pub enum ListFloorError {
    InternalServerError,
}

impl std::fmt::Display for ListFloorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ListFloorError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum UpdateFloorError {
    FloorNotFound(String),
    BuildingNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for UpdateFloorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateFloorError::FloorNotFound(message) => write!(f, "{}", message),
            UpdateFloorError::BuildingNotFound(message) => write!(f, "{}", message),
            UpdateFloorError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum DeleteFloorError {
    FloorNotFound(String),
    DeviceCascadeConstraint(String),
    InternalServerError,
}

impl std::fmt::Display for DeleteFloorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeleteFloorError::FloorNotFound(message) => write!(f, "{}", message),
            DeleteFloorError::DeviceCascadeConstraint(message) => write!(f, "{}", message),
            DeleteFloorError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}
