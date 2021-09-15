macro_rules! database_operation {
    ($connectionish:expr, $query:expr, $operation:ident) => {
        $crate::database::repository::macros::database_operation!(
            $connectionish,
            $query,
            $query,
            $operation
        )
    };
    ($connectionish:expr, $postgres_query:expr, $sqlite_query:expr, $operation:ident) => {
        $connectionish.with_connection(
            #[cfg(feature = "sqlite")]
            |connection| {
                $sqlite_query
                    .$operation(connection)
                    .map_err(RepositoryError::from)
            },
            #[cfg(feature = "postgres")]
            |connection| {
                $postgres_query
                    .$operation(connection)
                    .map_err(RepositoryError::from)
            },
        )
    };
}

#[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/4609
macro_rules! make_macro {
    ($DOLLAR:tt, $name:ident) => {
        macro_rules! $name {
            ($DOLLAR($args:tt)*) => {
                $crate::database::repository::macros::database_operation!(
                    $DOLLAR($args)*,
                    $name
                )
            };
        }
    };
}

make_macro!($, execute);
make_macro!($, load);
make_macro!($, first);
make_macro!($, get_results);

pub(crate) use database_operation;
pub(crate) use execute as execute_connection;
pub(crate) use execute as execute_pool;
pub(crate) use first as first_pool;
pub(crate) use get_results as get_results_pool;
pub(crate) use load as load_pool;
