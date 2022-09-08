use chrono::serde::ts_seconds_option;
use serde::{Deserialize, Serialize};

pub struct Request {
    pub id: i32,
    pub action: RequestActionType,
    pub announcement_id: i32,
    pub announcement_title: String,
    pub user_id: i32,
    pub user_name: String,
    pub description: String,
    pub approved_by_lsc: Option<bool>,
    pub lsc_approver: Option<i32>,
    pub approved_by_bm: Option<bool>,
    pub bm_approver: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct RequestMetadata {
    #[serde(with = "ts_seconds_option")]
    pub extended_end_date: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct RequestApproval {
    pub approved_by_lsc: Option<bool>,
    pub approved_by_bm: Option<bool>,
    pub lsc_approver: Option<i32>,
    pub bm_approver: Option<i32>,
}

#[derive(Debug, sqlx::Type, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "request_action_type", rename_all = "snake_case")]
pub enum RequestActionType {
    Create,
    ExtendDate,
    Delete,
    ChangeDevices,
}

impl RequestActionType {
    pub fn label(self) -> &'static str {
        match self {
            RequestActionType::Create => "Create",
            RequestActionType::ExtendDate => "Extend Date",
            RequestActionType::Delete => "Delete",
            RequestActionType::ChangeDevices => "Change Devices",
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            RequestActionType::Create => "create",
            RequestActionType::ExtendDate => "extend_date",
            RequestActionType::Delete => "delete",
            RequestActionType::ChangeDevices => "change_devices",
        }
    }
}

pub enum RequestErrorCode {
    EntityNotFound,
    RequestNotFound,
    UserNotFound,
    UserForbiddenToApprove,
    AnnouncementNotFound,
    RequestAlreadyApproved,
    InvalidAnnouncementStatus,
    InvalidExtendedEndDate,
    InternalServerError,
}

impl std::fmt::Display for RequestErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestErrorCode::EntityNotFound => write!(f, "ENTITY_NOT_FOUND"),
            RequestErrorCode::RequestNotFound => write!(f, "REQUEST_NOT_FOUND"),
            RequestErrorCode::UserNotFound => write!(f, "USER_NOT_FOUND"),
            RequestErrorCode::UserForbiddenToApprove => write!(f, "USER_FORBIDDEN_TO_APPROVE"),
            RequestErrorCode::AnnouncementNotFound => write!(f, "ANNOUNCEMENT_NOT_FOUND"),
            RequestErrorCode::RequestAlreadyApproved => write!(f, "REQUEST_ALREADY_APPROVED"),
            RequestErrorCode::InvalidAnnouncementStatus => write!(f, "INVALID_ANNOUNCEMENT_STATUS"),
            RequestErrorCode::InvalidExtendedEndDate => write!(f, "INVALID_EXTENDED_END_DATE"),
            RequestErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}

#[derive(Debug)]
pub enum CreateRequestError {
    EntityNotFound(String),
    AnnouncementNotFound(&'static str),
    InvalidExtendedEndDate(&'static str),
    InvalidAnnouncementStatus(&'static str),
    InternalServerError,
}

impl std::fmt::Display for CreateRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateRequestError::EntityNotFound(message) => write!(f, "{}", message),
            CreateRequestError::AnnouncementNotFound(message) => write!(f, "{}", message),
            CreateRequestError::InvalidExtendedEndDate(message) => write!(f, "{}", message),
            CreateRequestError::InvalidAnnouncementStatus(message) => write!(f, "{}", message),
            CreateRequestError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

#[derive(Debug)]
pub enum ListRequestError {
    InternalServerError,
}

impl std::fmt::Display for ListRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListRequestError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

#[derive(Debug)]
pub enum UpdateRequestApprovalError {
    RequestNotFound(String),
    UserNotFound(String),
    UserForbiddenToApprove(String),
    AnnouncementNotFound(String),
    RequestAlreadyApproved(String),
    InvalidAnnouncementStatus(String),
    InternalServerError,
}

impl std::fmt::Display for UpdateRequestApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateRequestApprovalError::RequestNotFound(message) => write!(f, "{}", message),
            UpdateRequestApprovalError::UserNotFound(message) => write!(f, "{}", message),
            UpdateRequestApprovalError::UserForbiddenToApprove(message) => write!(f, "{}", message),
            UpdateRequestApprovalError::AnnouncementNotFound(message) => write!(f, "{}", message),
            UpdateRequestApprovalError::RequestAlreadyApproved(message) => write!(f, "{}", message),
            UpdateRequestApprovalError::InvalidAnnouncementStatus(message) => {
                write!(f, "{}", message)
            }
            UpdateRequestApprovalError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum BatchRejectRequestsFromAnnouncementIdsError {
    InternalServerError,
}

impl std::fmt::Display for BatchRejectRequestsFromAnnouncementIdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchRejectRequestsFromAnnouncementIdsError::InternalServerError => {
                write!(f, "Internal Server Error")
            }
        }
    }
}
