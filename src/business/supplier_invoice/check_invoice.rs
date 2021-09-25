use super::{FullInvoice, InsertSupplierInvoiceError, UpdateSupplierInvoiceError};
use crate::{
    database::{
        repository::{FullInvoiceRepository, RepositoryError},
        schema::{InvoiceRow, InvoiceRowStatus, InvoiceRowType},
    },
    server::service::graphql::schema::types::InvoiceStatus,
};

pub async fn check_invoice_insert(
    repository: &FullInvoiceRepository,
    invoice_id: &str,
) -> Result<(), InsertSupplierInvoiceError> {
    use self::InsertSupplierInvoiceError::*;

    match repository.one(invoice_id).await {
        Ok(_) => Err(InvoiceExists),
        Err(error) => match &error {
            RepositoryError::NotFound => Ok(()),
            _ => Err(DBError(error)),
        },
    }
}

pub fn check_invoice_update(
    invoice_row: &InvoiceRow,
    new_status: &Option<InvoiceStatus>,
) -> Result<(), UpdateSupplierInvoiceError> {
    use self::UpdateSupplierInvoiceError::*;

    if invoice_row.r#type != InvoiceRowType::SupplierInvoice {
        return Err(NotASupplierInvoice);
    };

    if invoice_row.status == InvoiceRowStatus::Finalised {
        return Err(CannotEditFinalisedInvoice);
    };

    if let Some(InvoiceStatus::Draft) = new_status {
        if invoice_row.status != InvoiceRowStatus::Draft {
            return Err(CannoChangeInvoiceBackToDraft);
        }
    };

    // InvoiceDoesNotBelongToCurrentStore

    Ok(())
}

pub async fn get_invoice(
    repository: &FullInvoiceRepository,
    invoice_id: &str,
) -> Result<FullInvoice, UpdateSupplierInvoiceError> {
    use self::UpdateSupplierInvoiceError::*;

    match repository.one(invoice_id).await {
        Ok(invoice) => Ok(invoice),
        Err(error) => match &error {
            RepositoryError::NotFound => Err(InvoiceDoesNotExist),
            _ => Err(DBError(error)),
        },
    }
}
