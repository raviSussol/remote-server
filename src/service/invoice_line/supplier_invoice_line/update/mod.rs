use crate::{
    database::repository::{
        InvoiceLineRepository, RepositoryError, StockLineRepository, StorageConnectionManager,
    },
    domain::supplier_invoice::UpdateSupplierInvoiceLine,
};

mod generate;
mod validate;

use generate::generate;
use validate::validate;

pub fn update_supplier_invoice_line(
    connection_manager: &StorageConnectionManager,
    input: UpdateSupplierInvoiceLine,
) -> Result<String, UpdateSupplierInvoiceLineError> {
    let connection = connection_manager.connection()?;
    // TODO do inside transaction
    let (line, item, invoice) = validate(&input, &connection)?;

    let (updated_line, upsert_batch_option, delete_batch_id_option) =
        generate(input, line, item, invoice, &connection)?;

    InvoiceLineRepository::new(&connection).upsert_one(&updated_line)?;

    let stock_line_respository = StockLineRepository::new(&connection);

    if let Some(upsert_batch) = upsert_batch_option {
        stock_line_respository.upsert_one(&upsert_batch)?;
    }

    if let Some(id) = delete_batch_id_option {
        stock_line_respository.delete(&id)?;
    }

    Ok(updated_line.id)
}
pub enum UpdateSupplierInvoiceLineError {
    LineDoesNotExist,
    DatabaseError(RepositoryError),
    InvoiceDoesNotExist,
    NotASupplierInvoice,
    NotThisStoreInvoice,
    CannotEditFinalised,
    ItemNotFound,
    PackSizeBelowOne,
    NumberOfPacksBelowOne,
    BatchIsReserved,
    NotThisInvoiceLine(String),
}

impl From<RepositoryError> for UpdateSupplierInvoiceLineError {
    fn from(error: RepositoryError) -> Self {
        UpdateSupplierInvoiceLineError::DatabaseError(error)
    }
}