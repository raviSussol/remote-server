use async_graphql::Context;

use crate::{
    business::check_other_party_update,
    database::{
        repository::{
            FullInvoiceRepository, InvoiceRepository, NameQueryRepository, RepositoryError,
            StoreRepository,
        },
        schema::{InvoiceRowStatus, InvoiceRowType},
    },
    server::service::graphql::{
        schema::mutations::supplier_invoice::UpdateSupplierInvoiceInput, ContextExt,
    },
};

use super::{
    check_invoice_update, current_date_time, current_store_id, invoice_row, FullInvoice,
    UpdateSupplierInvoiceError,
};

impl From<RepositoryError> for UpdateSupplierInvoiceError {
    fn from(error: RepositoryError) -> Self {
        UpdateSupplierInvoiceError::DBError(error)
    }
}

pub async fn update_supplier_invoice(
    ctx: &Context<'_>,
    UpdateSupplierInvoiceInput {
        id,
        other_party_id,
        status,
        comment,
        their_reference,
    }: UpdateSupplierInvoiceInput,
) -> Result<(), UpdateSupplierInvoiceError> {
    let name_query_respository = ctx.get_repository::<NameQueryRepository>();
    let full_invoice_repository = ctx.get_repository::<FullInvoiceRepository>();
    let invoice_repository = ctx.get_repository::<InvoiceRepository>();
    let store_repository = ctx.get_repository::<StoreRepository>();

    let invoice_row = invoice_row(invoice_repository, &id).await?;
    check_invoice_update(&invoice_row, &status)?;
    check_other_party_update(name_query_respository, &other_party_id).await?;
    current_store_id(store_repository).await?;

    let status: Option<InvoiceRowStatus> = status.map(|status| status.into());
    let previous_status = invoice_row.status;

    let mut invoice = FullInvoice {
        id: invoice_row.id,
        r#type: InvoiceRowType::SupplierInvoice,
        store_id: invoice_row.store_id,
        invoice_number: invoice_row.invoice_number,
        confirm_datetime: invoice_row.confirm_datetime,
        finalised_datetime: invoice_row.finalised_datetime,
        entry_datetime: invoice_row.entry_datetime,
        status: previous_status.clone(),
        name_id: other_party_id.unwrap_or(invoice_row.name_id),
        comment: comment.or(invoice_row.comment),
        their_reference: their_reference.or(invoice_row.their_reference),
        // lines
    };

    set_new_status_datetime(&status, &previous_status, &mut invoice);

    if let Some(status) = status {
        invoice.status = status;
    }

    full_invoice_repository.update(invoice).await?;

    Ok(())
}

fn set_new_status_datetime(
    new_status: &Option<InvoiceRowStatus>,
    previous_status: &InvoiceRowStatus,
    invoice: &mut FullInvoice,
) {
    let current_datetime = current_date_time();

    use InvoiceRowStatus::*;

    if let Some(Finalised) = new_status {
        if *previous_status == Draft {
            invoice.confirm_datetime = Some(current_datetime.clone());
        }

        if *previous_status != Finalised {
            invoice.finalised_datetime = Some(current_datetime.clone());
        }
    }

    if let Some(Confirmed) = new_status {
        if *previous_status == Draft {
            invoice.confirm_datetime = Some(current_datetime.clone());
        }
    }
}
