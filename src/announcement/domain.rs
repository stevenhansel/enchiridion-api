pub enum AnnouncementErrorCode {
    InternalServerError,
}

impl std::fmt::Display for AnnouncementErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnnouncementErrorCode::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

pub enum CreateAnnouncementError {
    InternalServerError,
}

impl std::fmt::Display for CreateAnnouncementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateAnnouncementError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}
