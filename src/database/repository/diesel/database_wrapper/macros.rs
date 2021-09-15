macro_rules! database_operation {
    ($connectionish:expr, $query:expr, $(,)? $operation:ident) => {
        $crate::database::repository::macros::database_operation!(
            $connectionish,
            postgres => $query,
            sqlite => $query,
            $operation
        )
    };
    (
        $connectionish:expr,
        postgres => $postgres_query:expr,
        sqlite => $sqlite_query:expr,
        $(,)?
        $operation:ident
    ) => {
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
pub(crate) use database_operation;

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
        pub(crate) use $name;
    };
}

make_macro!($, execute);
make_macro!($, load);
make_macro!($, first);
make_macro!($, get_results);
