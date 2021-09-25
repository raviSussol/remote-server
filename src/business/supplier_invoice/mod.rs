use chrono::{NaiveDateTime, Utc};

use crate::{
    database::{
        repository::RepositoryError,
        schema::{InvoiceLineRow, InvoiceRow, StockLineRow},
    },
    server::service::graphql::schema::types::NameQuery,
};

pub mod check_invoice;
pub use self::check_invoice::*;

pub mod check_other_party;
pub use self::check_other_party::*;

pub mod check_store;
pub use self::check_store::*;

pub mod insert;
pub use self::insert::*;

pub mod update;
pub use self::update::*;

pub mod lines;
pub use self::lines::*;

pub struct FullInvoice {
    pub invoice: InvoiceRow,
    pub lines: Vec<FullInvoiceLine>,
}
pub struct FullInvoiceLine {
    pub line: InvoiceLineRow,
    pub batch: Option<StockLineRow>,
}

pub enum InsertSupplierInvoiceError {
    OtherPartyNotFound(String),
    OtherPartyIsNotASupplier(NameQuery),
    InvoiceExists,
    InvoiceLineErrors(Vec<InsertSupplierInvoiceLineErrors>),
    DBError(RepositoryError),
}

pub struct InsertSupplierInvoiceLineErrors {
    pub id: String,
    pub errors: Vec<InsertSupplierInvoiceLineError>,
}
pub enum InsertSupplierInvoiceLineError {
    PackSizeMustBeAboveOne(u32),
    SellPricePerPackMustBePositive(f64),
    CostPricePerPackMustBePositive(f64),
    InvoiceLineAlreadyExists,
    ItemIdNotFound(String),
}

pub enum UpdateSupplierInvoiceError {
    OtherPartyNotFound(String),
    OtherPartyIsNotASupplier(NameQuery),
    CannotEditFinalisedInvoice,
    InvoiceDoesNotExist,
    NotASupplierInvoice,
    InvoiceDoesNotBelongToCurrentStore,
    CannoChangeInvoiceBackToDraft,
    DBError(RepositoryError),
}

pub fn current_date_time() -> NaiveDateTime {
    Utc::now().naive_utc()
}
