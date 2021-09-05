use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    /// Row not found but expected at least one row
    #[error("row not found but expected at least one row")]
    NotFound,
    /// Row already exists
    #[error("row already exists")]
    UniqueViolation,
    /// Foreign key constraint is violated
    #[error("foreign key constraint is violated")]
    ForeignKeyViolation,
    /// Other DB related errors
    #[error("DBError: {msg:?}")]
    DBError { msg: String },
}

#[cfg_attr(feature = "mock", path = "mock/mod.rs")]
#[cfg_attr(feature = "pgsqlx", path = "pgsqlx/mod.rs")]
#[cfg_attr(
    any(feature = "dieselsqlite", feature = "dieselpg"),
    path = "diesel/mod.rs"
)]
#[cfg_attr(
    all(
        not(feature = "mock"),
        not(feature = "dieselsqlite"),
        not(feature = "dieselpg")
    ),
    path = "pgsqlx/mod.rs"
)]
pub mod repository;

pub use repository::{
    new_tx_name_repository, CustomerInvoiceRepository, DBTransaction, ItemLineRepository,
    ItemRepository, NameRepository, RequisitionLineRepository, RequisitionRepository,
    StoreRepository, TransactLineRepository, TransactRepository, TxManager, UserAccountRepository,
};
