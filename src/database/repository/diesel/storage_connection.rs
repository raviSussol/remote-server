use super::{get_connection, DBBackendConnection, DBConnection};

use crate::database::repository::RepositoryError;

use diesel::{
    connection::TransactionManager,
    r2d2::{ConnectionManager, Pool},
    Connection,
};
use futures_util::Future;

pub struct StorageConnection {
    pub connection: DBConnection,
}

pub enum TransactionError<E> {
    Transaction {
        msg: String,
    },
    /// Error from the transaction
    Inner(E),
}

impl From<TransactionError<RepositoryError>> for RepositoryError {
    fn from(error: TransactionError<RepositoryError>) -> Self {
        match error {
            TransactionError::Transaction { msg } => RepositoryError::DBError { msg },
            TransactionError::Inner(e) => e,
        }
    }
}

impl StorageConnection {
    pub async fn transaction<'a, T, E, F, Fut>(&'a self, f: F) -> Result<T, TransactionError<E>>
    where
        F: FnOnce(&'a StorageConnection) -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let con = &self.connection;
        let transaction_manager = con.transaction_manager();
        transaction_manager
            .begin_transaction(con)
            .map_err(|_| TransactionError::Transaction {
                msg: "Failed to start tx".to_string(),
            })?;

        match f(&self).await {
            Ok(value) => {
                transaction_manager.commit_transaction(con).map_err(|_| {
                    TransactionError::Transaction {
                        msg: "Failed to end tx".to_string(),
                    }
                })?;
                Ok(value)
            }
            Err(e) => {
                transaction_manager.rollback_transaction(con).map_err(|_| {
                    TransactionError::Transaction {
                        msg: "Failed to rollback tx".to_string(),
                    }
                })?;
                Err(TransactionError::Inner(e))
            }
        }
    }
}

pub struct StorageConnectionManager {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl StorageConnectionManager {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> Self {
        StorageConnectionManager { pool }
    }

    pub fn connection(&self) -> Result<StorageConnection, RepositoryError> {
        Ok(StorageConnection {
            connection: get_connection(&self.pool)?,
        })
    }
}