use super::InsertSupplierInvoiceError;
use crate::database::repository::{InvoiceRepository, RepositoryError};

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
