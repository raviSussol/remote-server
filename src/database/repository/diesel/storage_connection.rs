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

impl StorageConnection {
    pub async fn transaction<'a, T, F, Fut>(&'a self, f: F) -> Result<T, String>
    where
        F: FnOnce(&'a StorageConnection) -> Fut,
        Fut: Future<Output = Result<T, String>>,
    {
        let con = &self.connection;
        let transaction_manager = con.transaction_manager();
        transaction_manager
            .begin_transaction(con)
            .map_err(|_| "Failed to start tx".to_string())?;

        match f(&self).await {
            Ok(value) => {
                transaction_manager
                    .commit_transaction(con)
                    .map_err(|_| "Failed to end tx".to_string())?;
                Ok(value)
            }
            Err(e) => {
                transaction_manager
                    .rollback_transaction(con)
                    .map_err(|_| "Failed to rollback tx".to_string())?;
                Err(e)
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

    pub fn new_storage_context(&self) -> Result<StorageConnection, RepositoryError> {
        Ok(StorageConnection {
            connection: get_connection(&self.pool)?,
        })
    }
}

/*
#[cfg(test)]
mod transaction_manager_test {
    use crate::{database::schema::NameRow, util::test_db};

    use super::*;

    use diesel::prelude::*;

    pub struct NameRepository<'a> {
        connection: &'a StorageConnection,
    }



    #[actix_rt::test]
    async fn test() {
        let (pool, _, _) = test_db::setup_all("omsupply-database-transaction-manager", false).await;

        let manager = StorageContextManager::new(pool);
        let context = manager.new_storage_context().unwrap();
        context
            .transaction(|tx| async move {
                let repo = NameRepository::new(tx);
                let _ = repo.find_one_by_id("name_id").await.unwrap();
                let result = repo.find_many_by_id(&["ids".to_string()]).await.unwrap();
                Ok(result)
            })
            .await
            .unwrap();
    }
}
*/
