#[derive(Debug)]
pub enum DatabaseError {
    ForeignKeyError,
    UniqueConstraintError,
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DatabaseError::ForeignKeyError => write!(f, "23503"),
            DatabaseError::UniqueConstraintError => write!(f, "23505"),
        }
    }
}
