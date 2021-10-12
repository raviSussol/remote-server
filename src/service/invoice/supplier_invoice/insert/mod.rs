use crate::{
    database::repository::{InvoiceRepository, RepositoryError, StorageConnectionManager},
    domain::{name::Name, supplier_invoice::InsertSupplierInvoice},
};

mod generate;
mod validate;

use generate::generate;
use validate::validate;

use super::OtherPartyError;

pub fn insert_supplier_invoice(
    connection_manager: &StorageConnectionManager,
    input: InsertSupplierInvoice,
) -> Result<String, InsertSupplierInvoiceError> {
    let connection = connection_manager.connection()?;
    // TODO do inside transaction
    validate(&input, &connection)?;
    let new_invoice = generate(input, &connection)?;
    InvoiceRepository::new(&connection).upsert_one(&new_invoice)?;

    Ok(new_invoice.id)
}

pub enum InsertSupplierInvoiceError {
    InvoiceAlreadyExists,
    DatabaseError(RepositoryError),
    OtherPartyDoesNotExists,
    OtherPartyNotASupplier(Name),
}

impl From<RepositoryError> for InsertSupplierInvoiceError {
    fn from(error: RepositoryError) -> Self {
        InsertSupplierInvoiceError::DatabaseError(error)
    }
}

impl From<OtherPartyError> for InsertSupplierInvoiceError {
    fn from(error: OtherPartyError) -> Self {
        use InsertSupplierInvoiceError::*;
        match error {
            OtherPartyError::NotASupplier(name) => OtherPartyNotASupplier(name),
            OtherPartyError::DoesNotExist => OtherPartyDoesNotExists,
            OtherPartyError::DatabaseError(error) => DatabaseError(error),
        }
    }
}