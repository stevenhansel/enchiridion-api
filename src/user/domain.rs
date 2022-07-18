use serde::Serialize;

#[derive(Debug, sqlx::Type, PartialEq, Clone, Serialize)]
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
    pub password: Vec<u8>,
    pub registration_reason: Option<String>,

    pub is_email_confirmed: bool,
    pub status: UserStatus,
}
