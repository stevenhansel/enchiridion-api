#[derive(Debug, sqlx::Type, PartialEq, Clone)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
pub enum UserStatus {
    WaitingForApproval,
    Approved,
    Rejected,
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
