use serde::{Deserialize, Serialize};

pub struct Announcement {
    pub id: i32,
    pub title: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub status: AnnouncementStatus,
    pub user_id: i32,
    pub user_name: String,
    pub media: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, sqlx::Type, PartialEq, Clone, Serialize, Deserialize)]
#[sqlx(type_name = "announcement_status", rename_all = "snake_case")]
pub enum AnnouncementStatus {
    WaitingForApproval,
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
            AnnouncementStatus::Active => "Active",
            AnnouncementStatus::Done => "Done",
            AnnouncementStatus::Canceled => "Canceled",
            AnnouncementStatus::Rejected => "Rejected",
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            AnnouncementStatus::WaitingForApproval => "waiting_for_approval",
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
    UserNotFound,
    InternalServerError,
}

impl std::fmt::Display for AnnouncementErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
