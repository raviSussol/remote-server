use async_graphql::Context;
use chrono::NaiveDateTime;

use crate::{
    database::{
        repository::{
            FullInvoiceRepository, InvoiceLineRepository, ItemRepository, NameQueryRepository,
            RepositoryError, StoreRepository,
        },
        schema::{InvoiceRow, InvoiceRowType},
    },
    server::service::graphql::{
        schema::{mutations::supplier_invoice::InsertSupplierInvoiceInput, types::InvoiceStatus},
        ContextExt,
    },
};

use super::{
    check_invoice_insert, check_other_party_insert, current_date_time, current_store_id,
    get_insert_line_and_batches, FullInvoiceMutation, InsertSupplierInvoiceError, Mutations,
};

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
        lines,
    }: InsertSupplierInvoiceInput,
) -> Result<(), InsertSupplierInvoiceError> {
    let name_query_respository = ctx.get_repository::<NameQueryRepository>();
    let full_invoice_repository = ctx.get_repository::<FullInvoiceRepository>();
    let store_repository = ctx.get_repository::<StoreRepository>();
    let item_respository = ctx.get_repository::<ItemRepository>();
    let invoice_line_repository = ctx.get_repository::<InvoiceLineRepository>();

    check_invoice_insert(full_invoice_repository, &id).await?;
    check_other_party_insert(name_query_respository, &other_party_id).await?;

    let current_datetime = current_date_time();

    let invoice = InvoiceRow {
        id,
        comment,
        their_reference,
        r#type: InvoiceRowType::SupplierInvoice,
        store_id: current_store_id(store_repository).await?,
        name_id: other_party_id,
        invoice_number: new_invoice_number(),
        confirm_datetime: confirm_datetime(&status, &current_datetime),
        finalised_datetime: finalised_datetime(&status, &current_datetime),
        status: status.into(),
        entry_datetime: current_datetime,
    };

    let (lines, batches) =
        get_insert_line_and_batches(lines, invoice_line_repository, item_respository, &invoice)
            .await?;

    let full_invoice = FullInvoiceMutation {
        invoice: Mutations::new_inserts(invoice),
        lines,
        batches,
    };

    full_invoice_repository.mutate(full_invoice).await?;

    Ok(())
}

fn new_invoice_number() -> i32 {
    // TODO Need a mechanism for this
    1
}

fn confirm_datetime(status: &InvoiceStatus, current_time: &NaiveDateTime) -> Option<NaiveDateTime> {
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
