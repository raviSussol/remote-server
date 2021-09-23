use chrono::{NaiveDateTime, Utc};

use crate::{
    database::{
        repository::RepositoryError,
        schema::{InvoiceRowStatus, InvoiceRowType},
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

pub struct FullInvoice {
    pub id: String,
    pub name_id: String,
    pub store_id: String,
    pub invoice_number: i32,
    pub r#type: InvoiceRowType,
    pub status: InvoiceRowStatus,
    pub comment: Option<String>,
    pub their_reference: Option<String>,
    pub entry_datetime: NaiveDateTime,
    pub confirm_datetime: Option<NaiveDateTime>,
    pub finalised_datetime: Option<NaiveDateTime>,
    // lines
}

pub enum InsertSupplierInvoiceError {
    OtherPartyNotFound(String),
    OtherPartyIsNotASupplier(NameQuery),
    InvoiceExists,
    DBError(RepositoryError),
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
