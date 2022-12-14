use thiserror::Error;

pub enum LivestreamErrorCode {
    UnsupportedQuery,
    DatabaseError,
}

impl std::fmt::Display for LivestreamErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LivestreamErrorCode::UnsupportedQuery => write!(f, "UNSUPPORTED_QUERY"),
            LivestreamErrorCode::DatabaseError => write!(f, "DATABASE_ERROR"),
        }
    }
}

#[derive(Debug, Error)]
pub enum InsertLivestreamError {
    #[error("An error occurred with the request to the database")]
    DatabaseError(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum QueryLivestreamError {
    #[error("An error occurred with the request to the database")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Query is not supported")]
    UnsupportedQuery,
}
