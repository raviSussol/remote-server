use async_graphql::Context;

use crate::{
    business::check_other_party_update,
    database::{
        repository::{
            FullInvoiceRepository, NameQueryRepository, RepositoryError, StoreRepository,
        },
        schema::{InvoiceRow, InvoiceRowStatus},
    },
    server::service::graphql::{
        schema::mutations::supplier_invoice::UpdateSupplierInvoiceInput, ContextExt,
    },
};

use super::{
    check_invoice_update, current_date_time, current_store_id, get_invoice, FullInvoiceMutation,
    Mutations, UpdateSupplierInvoiceError,
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
        ..
    }: UpdateSupplierInvoiceInput,
) -> Result<(), UpdateSupplierInvoiceError> {
    let name_query_respository = ctx.get_repository::<NameQueryRepository>();
    let full_invoice_repository = ctx.get_repository::<FullInvoiceRepository>();
    let store_repository = ctx.get_repository::<StoreRepository>();

    let previous_invoice = get_invoice(full_invoice_repository, &id).await?;
    let mut previous_invoice_row = previous_invoice.invoice;

    check_invoice_update(&previous_invoice_row, &status)?;
    check_other_party_update(name_query_respository, &other_party_id).await?;
    current_store_id(store_repository).await?;

    let status: Option<InvoiceRowStatus> = status.map(|status| status.into());

    if let Some(other_party_id) = other_party_id {
        previous_invoice_row.name_id = other_party_id;
    }

    if let Some(comment) = comment {
        previous_invoice_row.comment = Some(comment);
    }

    if let Some(their_reference) = their_reference {
        previous_invoice_row.their_reference = Some(their_reference);
    }

    set_new_status_datetime(&status, &mut previous_invoice_row);

    if let Some(status) = status {
        previous_invoice_row.status = status;
    }

    let invoice = FullInvoiceMutation {
        invoice: Mutations::new_updates(previous_invoice_row),
        lines: Mutations::new(),
        batches: Mutations::new(),
    };

    full_invoice_repository.mutate(invoice).await?;

    Ok(())
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
