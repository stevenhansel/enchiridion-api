use serde::{Deserialize, Serialize};

use crate::features::media::domain::MediaType;

pub struct Announcement {
    pub id: i32,
    pub title: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub status: AnnouncementStatus,
    pub user_id: i32,
    pub user_name: String,
    pub media: String,
    pub media_type: MediaType,
    pub media_duration: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct AnnouncementDetail {
    pub id: i32,
    pub title: String,
    pub media: String,
    pub media_type: MediaType,
    pub media_duration: Option<f64>,
    pub notes: String,
    pub status: AnnouncementStatus,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub user_id: i32,
    pub user_name: String,
    pub devices: Vec<AnnouncementDetailDevices>,
}

pub struct AnnouncementDetailDevices {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub floor_id: i32,
}

pub struct AnnouncementMediaObject {
    pub filename: String,
    pub media: String,
}

#[derive(Debug, sqlx::Type, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "announcement_status", rename_all = "snake_case")]
pub enum AnnouncementStatus {
    WaitingForApproval,
    WaitingForSync,
    Active,
    Done,
    Canceled,
    Rejected,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementStatusObject {
    value: String,
    label: String,
}

impl AnnouncementStatus {
    pub fn label(self) -> &'static str {
        match self {
            AnnouncementStatus::WaitingForApproval => "Waiting For Approval",
            AnnouncementStatus::WaitingForSync => "Waiting For Sync",
            AnnouncementStatus::Active => "Active",
            AnnouncementStatus::Done => "Done",
            AnnouncementStatus::Canceled => "Canceled",
            AnnouncementStatus::Rejected => "Rejected",
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            AnnouncementStatus::WaitingForApproval => "waiting_for_approval",
            AnnouncementStatus::WaitingForSync => "waiting_for_sync",
            AnnouncementStatus::Active => "active",
            AnnouncementStatus::Done => "done",
            AnnouncementStatus::Canceled => "canceled",
            AnnouncementStatus::Rejected => "rejected",
        }
    }

    pub fn object(self) -> AnnouncementStatusObject {
        AnnouncementStatusObject {
            value: self.clone().value().to_string(),
            label: self.clone().label().to_string(),
        }
    }
}

pub enum AnnouncementErrorCode {
    AnnouncementNotFound,
    UserNotFound,
    InternalServerError,
}

impl std::fmt::Display for AnnouncementErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnnouncementErrorCode::AnnouncementNotFound => write!(f, "ANNOUNCEMENT_NOT_FOUND"),
            AnnouncementErrorCode::UserNotFound => write!(f, "USER_NOT_FOUND"),
            AnnouncementErrorCode::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
        }
    }
}

pub enum CreateAnnouncementError {
    UserNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for CreateAnnouncementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateAnnouncementError::UserNotFound(message) => write!(f, "{}", message),
            CreateAnnouncementError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum ListAnnouncementError {
    InternalServerError,
}

impl std::fmt::Display for ListAnnouncementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListAnnouncementError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum GetAnnouncementDetailError {
    AnnouncementNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for GetAnnouncementDetailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetAnnouncementDetailError::AnnouncementNotFound(message) => write!(f, "{}", message),
            GetAnnouncementDetailError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum GetAnnouncementMediaPresignedURLError {
    AnnouncementNotFound(String),
    InternalServerError,
}

impl std::fmt::Display for GetAnnouncementMediaPresignedURLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetAnnouncementMediaPresignedURLError::AnnouncementNotFound(message) => {
                write!(f, "{}", message)
            }
            GetAnnouncementMediaPresignedURLError::InternalServerError => {
                write!(f, "Internal Server Error")
            }
        }
    }
}

pub enum HandleScheduledAnnouncementsError {
    BrokenThread,
    InternalServerError,
}

impl std::fmt::Display for HandleScheduledAnnouncementsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleScheduledAnnouncementsError::BrokenThread => {
                write!(f, "Thread closed unexpectedly")
            },
            HandleScheduledAnnouncementsError::InternalServerError => {
                write!(f, "Internal Server Error")
            }
        }
    }
}

