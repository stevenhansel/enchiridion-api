use serde::{Deserialize, Serialize};

use crate::features::role::RoleObject;

#[derive(Debug, sqlx::Type, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
pub enum UserStatus {
    WaitingForApproval,
    Approved,
    Rejected,
}

impl UserStatus {
    pub fn label(self) -> &'static str {
        match self {
            UserStatus::WaitingForApproval => "Waiting For Approval",
            UserStatus::Approved => "Approved",
            UserStatus::Rejected => "Rejected",
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            UserStatus::WaitingForApproval => "waiting_for_approval",
            UserStatus::Approved => "approved",
            UserStatus::Rejected => "rejected",
        }
    }
}

pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: RoleObject,
    pub status: UserStatus,
    pub is_email_confirmed: bool,
    pub registration_reason: Option<String>,
    pub building: Option<UserBuilding>,
}

pub struct UserDetail {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: Vec<u8>,
    pub password_salt: String,
    pub registration_reason: Option<String>,
    pub is_email_confirmed: bool,
    pub status: UserStatus,
    pub building: Option<UserBuilding>,
}

pub struct UserBuilding {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub enum UserErrorCode {
    UserNotFound,
    UserNotConfirmed,
    UserStatusConflict,
    InternalServerError,
}

impl std::fmt::Display for UserErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserErrorCode::UserNotFound => write!(f, "USER_NOT_FOUND"),
            UserErrorCode::UserNotConfirmed => write!(f, "USER_NOT_CONFIRMED"),
            UserErrorCode::UserStatusConflict => write!(f, "USER_STATUS_CONFLICT"),
            UserErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}

pub enum ListUserError {
    InternalServerError,
}

impl std::fmt::Display for ListUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListUserError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum UpdateUserApprovalError {
    UserNotFound(&'static str),
    UserNotConfirmed(&'static str),
    UserStatusConflict(&'static str),
    InternalServerError,
}

impl std::fmt::Display for UpdateUserApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateUserApprovalError::UserNotFound(message) => write!(f, "{}", message),
            UpdateUserApprovalError::UserNotConfirmed(message) => write!(f, "{}", message),
            UpdateUserApprovalError::UserStatusConflict(message) => write!(f, "{}", message),
            UpdateUserApprovalError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}
