macro_rules! execute_connection {
    ($connection:expr, $query:expr) => {
        match $connection {
            crate::database::repository::DbConnection::Sqlite(connection) => {
                $query.execute(&*connection)
            }
            crate::database::repository::DbConnection::Pg(connection) => {
                $query.execute(&*connection)
            }
        }
    };
}

macro_rules! execute_pool {
    ($pool:expr, $query:expr) => {
        match $pool {
            crate::database::repository::DbConnectionPool::Sqlite(_) => $query
                .execute(&*$pool.get_sqlite_connection()?)
                .map_err(|err| RepositoryError::from(err)),
            crate::database::repository::DbConnectionPool::Pg(_) => $query
                .execute(&*$pool.get_pg_connection()?)
                .map_err(|err| RepositoryError::from(err)),
        }
    };
}

macro_rules! load_pool {
    ($pool:expr, $query:expr) => {
        match $pool {
            crate::database::repository::DbConnectionPool::Sqlite(_) => $query
                .load(&*$pool.get_sqlite_connection()?)
                .map_err(|err| RepositoryError::from(err)),
            crate::database::repository::DbConnectionPool::Pg(_) => $query
                .load(&*$pool.get_pg_connection()?)
                .map_err(|err| RepositoryError::from(err)),
        }
    };
}

macro_rules! first_pool {
    ($pool:expr, $query:expr) => {
        match $pool {
            crate::database::repository::DbConnectionPool::Sqlite(_) => $query
                .first(&*$pool.get_sqlite_connection()?)
                .map_err(|err| RepositoryError::from(err)),
            crate::database::repository::DbConnectionPool::Pg(_) => $query
                .first(&*$pool.get_pg_connection()?)
                .map_err(|err| RepositoryError::from(err)),
        }
    };
}

macro_rules! get_results_pool {
    ($pool:expr, $query:expr) => {
        match $pool {
            crate::database::repository::DbConnectionPool::Sqlite(_) => $query
                .get_results(&*$pool.get_sqlite_connection()?)
                .map_err(|err| RepositoryError::from(err)),
            crate::database::repository::DbConnectionPool::Pg(_) => $query
                .get_results(&*$pool.get_pg_connection()?)
                .map_err(|err| RepositoryError::from(err)),
        }
    };
}

macro_rules! transaction {
    ($connection:expr, $transaction_content:expr) => {
        match $connection {
            crate::database::repository::DbConnection::Sqlite(connection) => {
                connection.transaction($transaction_content)
            }
            crate::database::repository::DbConnection::Pg(connection) => {
                connection.transaction($transaction_content)
            }
        }
    };
}

pub(crate) use execute_connection;
pub(crate) use execute_pool;
pub(crate) use first_pool;
pub(crate) use get_results_pool;
pub(crate) use load_pool;
pub(crate) use transaction;
