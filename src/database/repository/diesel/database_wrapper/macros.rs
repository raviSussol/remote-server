#[cfg(feature = "postgres")]
macro_rules! postgres_operation {
    ($connection:expr, $query:expr, $operation:ident) => {
        $query
            .$operation($connection)
            .map_err(|err| RepositoryError::from(err))
    };
}

#[cfg(not(feature = "postgres"))]
macro_rules! postgres_operation {
    // How to match all ?
    ($connection:expr, $query:expr, $operation:ident) => {
        panic!("postgres flag is not enabled")
    };
}

#[cfg(feature = "sqlite")]
macro_rules! sqlite_operation {
    ($connection:expr, $query:expr, $operation:ident) => {
        $query
            .$operation($connection)
            .map_err(|err| RepositoryError::from(err))
    };
}

#[cfg(not(feature = "sqlite"))]
macro_rules! sqlite_operation {
    // How to match all ?
    ($connection:expr, $query:expr, $operation:ident) => {
        panic!("sqlite flag is not enabled")
    };
}

macro_rules! database_operation_pool {
    ($pool:expr, $postgres_query:expr, $sqlite_query:expr, $operation:ident) => {
        match $pool {
            #[allow(unused_variables)]
            crate::database::repository::DbConnectionPool::Sqlite(_) => {
                crate::database::repository::macros::sqlite_operation!(
                    &*$pool.get_sqlite_connection()?,
                    $sqlite_query,
                    $operation
                )
            }
            #[allow(unused_variables)]
            crate::database::repository::DbConnectionPool::Pg(_) => {
                crate::database::repository::macros::postgres_operation!(
                    &*$pool.get_pg_connection()?,
                    $postgres_query,
                    $operation
                )
            }
        }
    };
}

macro_rules! database_operation_connection {
    ($connection:expr, $postgres_query:expr, $sqlite_query:expr, $operation:ident) => {
        match $connection {
            #[allow(unused_variables)]
            crate::database::repository::DbConnection::Sqlite(connection) => {
                crate::database::repository::macros::sqlite_operation!(
                    &*connection,
                    $sqlite_query,
                    $operation
                )
            }
            #[allow(unused_variables)]
            crate::database::repository::DbConnection::Pg(connection) => {
                crate::database::repository::macros::postgres_operation!(
                    &*connection,
                    $postgres_query,
                    $operation
                )
            }
        }
    };
}

macro_rules! execute_connection {
    ($connection:expr, $query:expr) => {
        crate::database::repository::macros::database_operation_connection!(
            $connection,
            $query,
            $query,
            execute
        )
    };
    ($connection:expr, $postgres_query:expr,$sqlite_query:expr) => {
        crate::database::repository::macros::database_operation_connection!(
            $connection,
            $postgres_query,
            $sqlite_query,
            execute
        )
    };
}

macro_rules! execute_pool {
    ($connection:expr, $query:expr) => {
        crate::database::repository::macros::database_operation_pool!(
            $connection,
            $query,
            $query,
            execute
        )
    };
    ($connection:expr, $postgres_query:expr,$sqlite_query:expr) => {
        crate::database::repository::macros::database_operation_pool!(
            $connection,
            $postgres_query,
            $sqlite_query,
            execute
        )
    };
}

macro_rules! load_pool {
    ($connection:expr, $query:expr) => {
        crate::database::repository::macros::database_operation_pool!(
            $connection,
            $query,
            $query,
            load
        )
    };
    ($connection:expr, $postgres_query:expr,$sqlite_query:expr) => {
        crate::database::repository::macros::database_operation_pool!(
            $connection,
            $postgres_query,
            $sqlite_query,
            load
        )
    };
}

macro_rules! first_pool {
    ($connection:expr, $query:expr) => {
        crate::database::repository::macros::database_operation_pool!(
            $connection,
            $query,
            $query,
            first
        )
    };
    ($connection:expr, $postgres_query:expr,$sqlite_query:expr) => {
        crate::database::repository::macros::database_operation_pool!(
            $connection,
            $postgres_query,
            $sqlite_query,
            first
        )
    };
}

macro_rules! get_results_pool {
    ($connection:expr, $query:expr) => {
        crate::database::repository::macros::database_operation_pool!(
            $connection,
            $query,
            $query,
            get_results
        )
    };
    ($connection:expr, $postgres_query:expr,$sqlite_query:expr) => {
        crate::database::repository::macros::database_operation_pool!(
            $connection,
            $postgres_query,
            $sqlite_query,
            get_results
        )
    };
}

#[cfg(feature = "postgres")]
macro_rules! postgres_transaction {
    ($expression:expr) => {
        $expression
    };
}

#[cfg(not(feature = "postgres"))]
macro_rules! postgres_transaction {
    // How to match all ?
    ($expression:expr) => {
        panic!("postgres flag is not enabled")
    };
}

#[cfg(feature = "sqlite")]
macro_rules! sqlite_transaction {
    ($expression:expr) => {
        $expression
    };
}

#[cfg(not(feature = "sqlite"))]
macro_rules! sqlite_transaction {
    // How to match all ?
    ($expression:expr) => {
        panic!("sqlite flag is not enabled")
    };
}

macro_rules! transaction {
    ($connection:expr, $transaction_content:expr) => {
        match $connection {
            #[allow(unused_variables)]
            crate::database::repository::DbConnection::Sqlite(connection) => {
                crate::database::repository::macros::sqlite_transaction!(
                    connection.transaction($transaction_content)
                )
            }
            #[allow(unused_variables)]
            crate::database::repository::DbConnection::Pg(connection) => {
                crate::database::repository::macros::postgres_transaction!(
                    connection.transaction($transaction_content)
                )
            }
        }
    };
}

pub(crate) use database_operation_connection;
pub(crate) use database_operation_pool;
pub(crate) use execute_connection;
pub(crate) use execute_pool;
pub(crate) use first_pool;
pub(crate) use get_results_pool;
pub(crate) use load_pool;
pub(crate) use postgres_operation;
pub(crate) use postgres_transaction;
pub(crate) use sqlite_operation;
pub(crate) use sqlite_transaction;
pub(crate) use transaction;
