use crate::database::repository::RepositoryError;

use super::{get_connection, DBBackendConnection, DBTransaction};

use diesel::{
    connection::TransactionManager,
    r2d2::{ConnectionManager, Pool},
    Connection,
};

#[derive(Clone)]
pub struct TxManager {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl TxManager {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> TxManager {
        TxManager { pool }
    }

    pub fn create_tx(&self) -> Result<DBTransaction, RepositoryError> {
        get_connection(&self.pool)
    }

    pub async fn transaction<'a, T, Fut>(
        &self,
        con: &DBTransaction,
        f: impl FnOnce() -> Fut,
    ) -> Result<T, RepositoryError>
    where
        Fut: std::future::Future<Output = Result<T, RepositoryError>>,
    {
        //let con = get_connection(&self.pool)?;
        let transaction_manager = con.transaction_manager();
        transaction_manager.begin_transaction(con)?;

        match f().await {
            Ok(value) => {
                transaction_manager.commit_transaction(con)?;
                Ok(value)
            }
            Err(e) => {
                transaction_manager.rollback_transaction(con)?;
                Err(e)
            }
        }
    }
}
