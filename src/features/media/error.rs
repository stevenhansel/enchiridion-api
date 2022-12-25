use thiserror::Error;

pub enum MediaErrorCode {
    InternalServerError,
}

impl std::fmt::Display for MediaErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MediaErrorCode::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

#[derive(Debug, Error)]
pub enum CreateMediaError {
    #[error("An error occurred with the request to the database")]
    Database(#[from] sqlx::Error),

    #[error("An unknown error has occurred")]
    Unknown,
}
