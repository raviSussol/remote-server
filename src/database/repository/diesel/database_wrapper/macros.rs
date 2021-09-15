macro_rules! database_operation_pool {
    ($pool:expr, $postgres_query:expr, $sqlite_query:expr, $operation:ident) => {
        match &$pool {
            #[cfg(feature = "sqlite")]
            #[allow(unused_variables)]
            $crate::database::repository::DbConnectionPool::Sqlite(pool) => $sqlite_query
                .$operation(&*pool.get()?)
                .map_err(RepositoryError::from),
            #[cfg(feature = "postgres")]
            #[allow(unused_variables)]
            $crate::database::repository::DbConnectionPool::Pg(pool) => $postgres_query
                .$operation(&*pool.get()?)
                .map_err(RepositoryError::from),
        }
    };
}

macro_rules! database_operation_connection {
    ($connection:expr, $postgres_query:expr, $sqlite_query:expr, $operation:ident) => {
        match $connection {
            #[cfg(feature = "sqlite")]
            #[allow(unused_variables)]
            $crate::database::repository::DbConnection::Sqlite(connection) => $sqlite_query
                .$operation(&*connection)
                .map_err(RepositoryError::from),
            #[cfg(feature = "postgres")]
            #[allow(unused_variables)]
            $crate::database::repository::DbConnection::Pg(connection) => $postgres_query
                .$operation(&*connection)
                .map_err(RepositoryError::from),
        }
    };
}

macro_rules! execute_connection {
    ($connection:expr, $query:expr) => {
        $crate::database::repository::macros::database_operation_connection!(
            $connection,
            $query,
            $query,
            execute
        )
    };
    ($connection:expr, $postgres_query:expr, $sqlite_query:expr) => {
        $crate::database::repository::macros::database_operation_connection!(
            $connection,
            $postgres_query,
            $sqlite_query,
            execute
        )
    };
}

macro_rules! execute_pool {
    ($connection:expr, $query:expr) => {
        $crate::database::repository::macros::database_operation_pool!(
            $connection,
            $query,
            $query,
            execute
        )
    };
    ($connection:expr, $postgres_query:expr,$sqlite_query:expr) => {
        $crate::database::repository::macros::database_operation_pool!(
            $connection,
            $postgres_query,
            $sqlite_query,
            execute
        )
    };
}

macro_rules! load_pool {
    ($connection:expr, $query:expr) => {
        $crate::database::repository::macros::database_operation_pool!(
            $connection,
            $query,
            $query,
            load
        )
    };
    ($connection:expr, $postgres_query:expr,$sqlite_query:expr) => {
        $crate::database::repository::macros::database_operation_pool!(
            $connection,
            $postgres_query,
            $sqlite_query,
            load
        )
    };
}

macro_rules! first_pool {
    ($connection:expr, $query:expr) => {
        $crate::database::repository::macros::database_operation_pool!(
            $connection,
            $query,
            $query,
            first
        )
    };
    ($connection:expr, $postgres_query:expr,$sqlite_query:expr) => {
        $crate::database::repository::macros::database_operation_pool!(
            $connection,
            $postgres_query,
            $sqlite_query,
            first
        )
    };
}

macro_rules! get_results_pool {
    ($connection:expr, $query:expr) => {
        $crate::database::repository::macros::database_operation_pool!(
            $connection,
            $query,
            $query,
            get_results
        )
    };
    ($connection:expr, $postgres_query:expr,$sqlite_query:expr) => {
        $crate::database::repository::macros::database_operation_pool!(
            $connection,
            $postgres_query,
            $sqlite_query,
            get_results
        )
    };
}

pub(crate) use database_operation_connection;
pub(crate) use database_operation_pool;
pub(crate) use execute_connection;
pub(crate) use execute_pool;
pub(crate) use first_pool;
pub(crate) use get_results_pool;
pub(crate) use load_pool;
