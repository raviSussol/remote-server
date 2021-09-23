use crate::{
    database::{
        repository::{
            FullInvoice, FullInvoiceRepository, InvoiceRepository, NameQueryRepository,
            RepositoryError, StoreRepository,
        },
        schema::{InvoiceRow, InvoiceRowType},
    },
    server::service::graphql::{
        schema::{
            mutations::supplier_invoice::InsertSupplierInvoiceInput,
            types::{InvoiceStatus, NameQuery},
        },
        ContextExt,
    },
};
use async_graphql::*;
use chrono::{NaiveDateTime, Utc};

pub mod check_invoice;
pub use self::check_invoice::*;

pub mod check_other_party;
pub use self::check_other_party::*;

pub mod check_store;
pub use self::check_store::*;

pub enum InsertSupplierInvoiceError {
    OtherPartyNotFound(String),
    OtherPartyIsNotASupplier(NameQuery),
    InvoiceExists,
    DBError(RepositoryError),
}

impl From<RepositoryError> for InsertSupplierInvoiceError {
    fn from(error: RepositoryError) -> Self {
        InsertSupplierInvoiceError::DBError(error)
    }
}

pub async fn insert_supplier_invoice(
    ctx: &Context<'_>,
    InsertSupplierInvoiceInput {
        id,
        other_party_id,
        status,
        comment,
        their_reference,
    }: InsertSupplierInvoiceInput,
) -> Result<InvoiceRow, InsertSupplierInvoiceError> {
    let name_query_respository = ctx.get_repository::<NameQueryRepository>();
    let full_invoice_repository = ctx.get_repository::<FullInvoiceRepository>();
    let invoice_repository = ctx.get_repository::<InvoiceRepository>();
    let store_repository = ctx.get_repository::<StoreRepository>();

    check_invoice_insert(invoice_repository, &id).await?;
    check_other_party(name_query_respository, &other_party_id).await?;

    let current_datetime = current_date_time();

    let invoice = FullInvoice {
        id,
        comment,
        their_reference,
        r#type: InvoiceRowType::SupplierInvoice,
        store_id: current_store_id(store_repository).await?,
        name_id: other_party_id,
        invoice_number: new_invoice_number(),
        confirm_datetime: config_datetime(&status, &current_datetime),
        finalised_datetime: finalised_datetime(&status, &current_datetime),
        status: status.into(),
        entry_datetime: current_datetime,
    };

    Ok(full_invoice_repository.insert(invoice).await?)
}

fn current_date_time() -> NaiveDateTime {
    Utc::now().naive_utc()
}

fn new_invoice_number() -> i32 {
    // TODO Need a mechanism for this
    1
}

fn config_datetime(status: &InvoiceStatus, current_time: &NaiveDateTime) -> Option<NaiveDateTime> {
    match status {
        InvoiceStatus::Draft => None,
        _ => Some(current_time.clone()),
    }
}

fn finalised_datetime(
    status: &InvoiceStatus,
    current_time: &NaiveDateTime,
) -> Option<NaiveDateTime> {
    match status {
        InvoiceStatus::Draft => None,
        _ => Some(current_time.clone()),
    }
}
