use super::{InsertSupplierInvoiceError, UpdateSupplierInvoiceError};
use crate::{
    database::{
        repository::{InvoiceRepository, RepositoryError},
        schema::{InvoiceRow, InvoiceRowStatus, InvoiceRowType},
    },
    server::service::graphql::schema::types::InvoiceStatus,
};

pub async fn check_invoice_insert(
    invoice_respository: &InvoiceRepository,
    invoice_id: &str,
) -> Result<(), InsertSupplierInvoiceError> {
    use self::InsertSupplierInvoiceError::*;

    match invoice_respository.find_one_by_id(invoice_id).await {
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

pub async fn invoice_row(
    invoice_respository: &InvoiceRepository,
    invoice_id: &str,
) -> Result<InvoiceRow, UpdateSupplierInvoiceError> {
    use self::UpdateSupplierInvoiceError::*;

    match invoice_respository.find_one_by_id(invoice_id).await {
        Ok(invoice_row) => Ok(invoice_row),
        Err(error) => match &error {
            RepositoryError::NotFound => Err(InvoiceDoesNotExist),
            _ => Err(DBError(error)),
        },
    }
}
