use thiserror::Error;

#[derive(Debug, Error)]
pub enum InsertLivestreamError {
    #[error("An error occurred with the request to the database")]
    DatabaseError(#[from] sqlx::Error),
}
