use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Device {
    pub id: i32,
    pub name: String,
    pub location: String,
    pub description: String,
    pub active_announcements: i32,
}

#[derive(Debug)]
pub struct DeviceDetail {
    pub id: i32,
    pub name: String,
    pub location: DeviceDetailLocation,
    pub floor_id: i32,
    pub building_id: i32,
    pub description: String,
    pub camera_enabled: bool,
    pub active_announcements: i32,
    pub access_key_id: String,
    pub secret_access_key: Vec<u8>,
    pub secret_access_key_salt: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub linked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub struct DeviceDetailLocation {
    pub text: String, 
    pub building_id: i32,
    pub building_name: String,
    pub building_color: String,
    pub floor_id: i32,
    pub floor_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAuthCache {
    pub device_id: i32,
    pub secret_access_key: String,
    pub secret_access_key_salt: String,
    pub linked_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl DeviceAuthCache {
    pub fn new(device_id: i32, secret_access_key: String, secret_access_key_salt: String, linked_at: Option<chrono::DateTime<chrono::Utc>>) -> Self {
        DeviceAuthCache {
            device_id,
            secret_access_key,
            secret_access_key_salt,
            linked_at,
        }
    }
}

pub struct ListDeviceParams {
    pub page: i32,
    pub limit: i32,
    pub query: Option<String>,
    pub building_id: Option<i32>,
    pub floor_id: Option<i32>,
}

pub enum DeviceErrorCode {
    AuthenticationFailed,
    DeviceAlreadyExists,
    DeviceNotFound,
    FloorNotFound,
    DeviceCascadeConstraint,
    DeviceLinkingError,
    InternalServerError,
}

impl std::fmt::Display for DeviceErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceErrorCode::AuthenticationFailed => write!(f, "AUTHENTICATION_FAILED"),
            DeviceErrorCode::DeviceAlreadyExists => write!(f, "DEVICE_ALREADY_EXISTS"),
            DeviceErrorCode::DeviceNotFound => write!(f, "DEVICE_NOT_FOUND"),
            DeviceErrorCode::FloorNotFound => write!(f, "BUILDING_NOT_FOUND"),
            DeviceErrorCode::DeviceCascadeConstraint => write!(f, "DEVICE_CASCADE_CONSTRAINT"),
            DeviceErrorCode::DeviceLinkingError => write!(f, "DEVICE_LINKING_ERROR"),
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
    DeviceCascadeConstraint(String),
    InternalServerError,
}

impl std::fmt::Display for DeleteDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeleteDeviceError::DeviceNotFound(message) => write!(f, "{}", message),
            DeleteDeviceError::DeviceCascadeConstraint(message) => write!(f, "{}", message),
            DeleteDeviceError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum ResyncDeviceError {
    DeviceNotFound(&'static str),
    InternalServerError,
}

impl std::fmt::Display for ResyncDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResyncDeviceError::DeviceNotFound(message) => write!(f, "{}", message),
            ResyncDeviceError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AuthenticateDeviceError {
    AuthenticationFailed(&'static str),
    DeviceNotFound(&'static str),
    InternalServerError,
}

impl std::fmt::Display for AuthenticateDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthenticateDeviceError::AuthenticationFailed(message) => write!(f, "{}", message),
            AuthenticateDeviceError::DeviceNotFound(message) => write!(f, "{}", message),
            AuthenticateDeviceError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum GetDeviceDetailByAccessKeyIdError {
    DeviceNotFound(&'static str),
    InternalServerError,
}

impl std::fmt::Display for GetDeviceDetailByAccessKeyIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GetDeviceDetailByAccessKeyIdError::DeviceNotFound(message) => write!(f, "{}", message),
            GetDeviceDetailByAccessKeyIdError::InternalServerError => {
                write!(f, "Internal Server Error")
            }
        }
    }
}

pub enum GetDeviceAuthCacheError {
    DeviceNotFound(&'static str),
    InternalServerError,
}

impl std::fmt::Display for GetDeviceAuthCacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GetDeviceAuthCacheError::DeviceNotFound(message) => write!(f, "{}", message),
            GetDeviceAuthCacheError::InternalServerError => {
                write!(f, "Internal Server Error")
            }
        }
    }
}

pub enum LinkDeviceError {
    AuthenticationFailed(&'static str),
    DeviceNotFound(&'static str),
    DeviceLinkingError(&'static str),
    InternalServerError,
}

impl std::fmt::Display for LinkDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinkDeviceError::AuthenticationFailed(message) => write!(f, "{}", message),
            LinkDeviceError::DeviceNotFound(message) => write!(f, "{}", message),
            LinkDeviceError::DeviceLinkingError(message) => write!(f, "{}", message),
            LinkDeviceError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum UnlinkDeviceError {
    DeviceNotFound(&'static str),
    DeviceLinkingError(&'static str),
    InternalServerError,
}

impl std::fmt::Display for UnlinkDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnlinkDeviceError::DeviceNotFound(message) => write!(f, "{}", message),
            UnlinkDeviceError::DeviceLinkingError(message) => write!(f, "{}", message),
            UnlinkDeviceError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}
