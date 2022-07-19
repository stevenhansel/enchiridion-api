use core::fmt;

#[derive(Debug)]
pub struct Device {
    pub id: i32,
    pub name: String,
    pub location: String,
    pub description: String,
}

pub struct ListDeviceParams {
    pub page: i32,
    pub limit: i32,
    pub query: Option<String>,
    pub building_id: Option<i32>,
    pub floor_id: Option<i32>,
}

pub enum DeviceErrorCode {
    DeviceAlreadyExists,
    FloorNotFound,
    InternalServerError,
}

impl std::fmt::Display for DeviceErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceErrorCode::DeviceAlreadyExists => write!(f, "DEVICE_ALREADY_EXISTS"),
            DeviceErrorCode::FloorNotFound => write!(f, "BUILDING_NOT_FOUND"),
            DeviceErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}

pub enum CreateDeviceError {
    DeviceAlreadyExists(String),
    FloorNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for CreateDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CreateDeviceError::DeviceAlreadyExists(message) => write!(f, "{}", message),
            CreateDeviceError::FloorNotFound(message) => write!(f, "{}", message),
            CreateDeviceError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum ListDeviceError {
    InternalServerError,
}

impl std::fmt::Display for ListDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ListDeviceError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}
