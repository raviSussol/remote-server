use async_graphql::ErrorExtensions;
use repository::RepositoryError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Internal error")]
    InternalError(String),

    #[error("Bad user input")]
    BadUserInput(String),

    #[error("Unauthenticated")]
    Unauthenticated(String),

    #[error("Forbidden")]
    Forbidden(String),
}

impl ErrorExtensions for ServerError {
    // lets define our base extensions
    fn extend(self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_, e| {
            e.set("code", format!("{:?}", self));
            match self {
                ServerError::InternalError(details) => e.set("details", details),
                ServerError::BadUserInput(details) => e.set("details", details),
                ServerError::Unauthenticated(details) => e.set("details", details),
                ServerError::Forbidden(details) => e.set("details", details),
            }
        })
    }
}

impl From<RepositoryError> for ServerError {
    fn from(err: RepositoryError) -> Self {
        ServerError::InternalError(format!("{:?}", err))
    }
}
