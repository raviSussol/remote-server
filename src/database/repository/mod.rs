use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum RepositoryError {
    /// Row not found but expected at least one row
    #[error("row not found but expected at least one row")]
    NotFound,
    /// Row already exists
    #[error("row already exists")]
    UniqueViolation,
    // Connection doesn't exist
    #[error("connection does not exist")]
    ConnectionDoesntExist(ConnectionType),
    // Other connection error
    #[error("r2d2 connection error")]
    OtherConnectionError(String),
    /// Foreign key constraint is violated
    #[error("foreign key constraint is violated")]
    ForeignKeyViolation,
    /// Other DB related errors
    #[error("DBError: {msg:?}")]
    DBError { msg: String },
}

pub mod diesel;
pub use self::diesel::*;
