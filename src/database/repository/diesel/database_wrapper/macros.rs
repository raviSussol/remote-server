macro_rules! database_operation_pool {
    ($pool:expr, $query:expr, $operation:ident) => {
        $crate::database::repository::macros::database_operation_pool!(
            $pool, $query, $query, $operation
        )
    };
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
    ($connection:expr, $query:expr, $operation:ident) => {
        $crate::database::repository::macros::database_operation_connection!(
            $connection,
            $query,
            $query,
            $operation
        )
    };
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

#[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/4609
macro_rules! make_macro {
    ($DOLLAR:tt, $macro_name:ident, $call_macro:ident, $arg:ident) => {
        macro_rules! $macro_name {
            ($DOLLAR($args:tt)*) => {
                $crate::database::repository::macros::$call_macro!(
                    $DOLLAR($args)*,
                    $arg
                )
            };
        }
    };
}

make_macro!($, execute_connection, database_operation_connection, execute);
make_macro!($, execute_pool, database_operation_pool, execute);
make_macro!($, load_pool, database_operation_pool, load);
make_macro!($, first_pool, database_operation_pool, first);
make_macro!($, get_results_pool, database_operation_pool, load);

pub(crate) use database_operation_connection;
pub(crate) use database_operation_pool;
pub(crate) use execute_connection;
pub(crate) use execute_pool;
pub(crate) use first_pool;
pub(crate) use get_results_pool;
pub(crate) use load_pool;
