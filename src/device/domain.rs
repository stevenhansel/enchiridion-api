use core::fmt;

#[derive(Debug)]
pub struct Device {}

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
