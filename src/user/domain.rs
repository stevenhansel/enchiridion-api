use serde::{Serialize, Deserialize};

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
    pub role_id: i32,
    pub role_name: String,
    pub status: UserStatus,
    pub registration_reason: Option<String>,
}

pub struct UserDetail {
    pub id: i32,

    pub name: String,
    pub email: String,
    pub password: Vec<u8>,
    pub registration_reason: Option<String>,

    pub is_email_confirmed: bool,
    pub status: UserStatus,
}

pub enum UserErrorCode {
    InternalServerError,
}

impl std::fmt::Display for UserErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
