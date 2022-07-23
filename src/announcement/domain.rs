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
