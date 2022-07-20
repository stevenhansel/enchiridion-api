use core::fmt;

#[derive(Debug)]
pub struct Device {
    pub id: i32,
    pub name: String,
    pub location: String,
    pub description: String,
}

#[derive(Debug)]
pub struct DeviceDetail {
    pub id: i32,
    pub name: String,
    pub location: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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
    DeviceNotFound,
    FloorNotFound,
    InternalServerError,
}

impl std::fmt::Display for DeviceErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceErrorCode::DeviceAlreadyExists => write!(f, "DEVICE_ALREADY_EXISTS"),
            DeviceErrorCode::DeviceNotFound => write!(f, "DEVICE_NOT_FOUND"),
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

pub enum GetDeviceDetailByIdError {
    DeviceNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for GetDeviceDetailByIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GetDeviceDetailByIdError::DeviceNotFound(message) => write!(f, "{}", message),
            GetDeviceDetailByIdError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum UpdateDeviceError {
    DeviceNotFound(String),
    DeviceAlreadyExists(String),
    FloorNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for UpdateDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateDeviceError::DeviceNotFound(message) => write!(f, "{}", message),
            UpdateDeviceError::DeviceAlreadyExists(message) => write!(f, "{}", message),
            UpdateDeviceError::FloorNotFound(message) => write!(f, "{}", message),
            UpdateDeviceError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum DeleteDeviceError {
    DeviceNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for DeleteDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeleteDeviceError::DeviceNotFound(message) => write!(f, "{}", message),
            DeleteDeviceError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

