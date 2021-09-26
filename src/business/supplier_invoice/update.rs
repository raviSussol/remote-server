use async_graphql::Context;

use crate::{
    business::check_other_party_update,
    database::{
        repository::{
            FullInvoiceRepository, InvoiceLineRepository, ItemRepository, NameQueryRepository,
            StoreRepository,
        },
        schema::{InvoiceRow, InvoiceRowStatus},
    },
    server::service::graphql::{
        schema::mutations::supplier_invoice::UpdateSupplierInvoiceInput, ContextExt,
    },
};

use super::{
    check_invoice_update, current_date_time, current_store_id, get_invoice,
    get_upsert_lines_and_batches, FullInvoice, FullInvoiceMutation, Mutations,
    UpdateSupplierInvoiceError,
};

pub async fn update_supplier_invoice(
    ctx: &Context<'_>,
    input: UpdateSupplierInvoiceInput,
) -> Result<(), UpdateSupplierInvoiceError> {
    let name_query_respository = ctx.get_repository::<NameQueryRepository>();
    let full_invoice_repository = ctx.get_repository::<FullInvoiceRepository>();
    let store_repository = ctx.get_repository::<StoreRepository>();
    let item_respository = ctx.get_repository::<ItemRepository>();
    let invoice_line_repository = ctx.get_repository::<InvoiceLineRepository>();

    let previous_invoice = get_invoice(full_invoice_repository, &input.id).await?;

    check_invoice_update(&previous_invoice.invoice, &input.status)?;
    check_other_party_update(name_query_respository, &input.other_party_id).await?;
    current_store_id(store_repository).await?;

    let invoice_mutation = get_updated_invoice(
        previous_invoice,
        input,
        invoice_line_repository,
        item_respository,
    )
    .await?;

    full_invoice_repository.mutate(invoice_mutation).await?;

    Ok(())
}

pub async fn get_updated_invoice(
    previous_invoice: FullInvoice,
    UpdateSupplierInvoiceInput {
        id: _,
        other_party_id,
        status,
        comment,
        their_reference,
        lines,
    }: UpdateSupplierInvoiceInput,
    invoice_line_repository: &InvoiceLineRepository,
    item_respository: &ItemRepository,
) -> Result<FullInvoiceMutation, UpdateSupplierInvoiceError> {
    let status: Option<InvoiceRowStatus> = status.map(|status| status.into());

    let mut updated_invoice = previous_invoice.invoice;

    if let Some(other_party_id) = other_party_id {
        updated_invoice.name_id = other_party_id;
    }

    if let Some(comment) = comment {
        updated_invoice.comment = Some(comment);
    }

    if let Some(their_reference) = their_reference {
        updated_invoice.their_reference = Some(their_reference);
    }

    set_new_status_datetime(&status, &mut updated_invoice);

    if let Some(status) = status {
        updated_invoice.status = status;
    }

    let (lines, batches) = get_upsert_lines_and_batches(
        lines,
        invoice_line_repository,
        item_respository,
        &updated_invoice,
        previous_invoice.lines,
    )
    .await?;

    Ok(FullInvoiceMutation {
        invoice: Mutations::new_updates(updated_invoice),
        lines,
        batches,
    })
}

fn set_new_status_datetime(
    new_status: &Option<InvoiceRowStatus>,
    previous_invoice: &mut InvoiceRow,
) {
    let current_datetime = current_date_time();

    use InvoiceRowStatus::*;

    if let Some(Finalised) = new_status {
        if previous_invoice.status == Draft {
            previous_invoice.confirm_datetime = Some(current_datetime.clone());
        }

        if previous_invoice.status != Finalised {
            previous_invoice.finalised_datetime = Some(current_datetime.clone());
        }
    }

    if let Some(Confirmed) = new_status {
        if previous_invoice.status == Draft {
            previous_invoice.confirm_datetime = Some(current_datetime.clone());
        }
    }
}
