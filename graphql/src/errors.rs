use async_graphql::ErrorExtensions;
use repository::RepositoryError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StandardError {
    #[error("Internal error")]
    InternalError(String),

    #[error("Bad user input")]
    BadUserInput(String),

    #[error("Unauthenticated")]
    Unauthenticated(String),

    #[error("Forbidden")]
    Forbidden(String),
}

impl ErrorExtensions for StandardError {
    // lets define our base extensions
    fn extend(self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_, e| {
            e.set("code", format!("{:?}", self));
            match self {
                StandardError::InternalError(details) => e.set("details", details),
                StandardError::BadUserInput(details) => e.set("details", details),
                StandardError::Unauthenticated(details) => e.set("details", details),
                StandardError::Forbidden(details) => e.set("details", details),
            }
        })
    }
}

impl From<RepositoryError> for StandardError {
    fn from(err: RepositoryError) -> Self {
        StandardError::InternalError(format!("{:?}", err))
    }
}
