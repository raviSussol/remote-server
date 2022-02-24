use crate::{invoice_line::DeleteOutboundShipmentLine, WithDBError};
use repository::{
    InvoiceLineRowRepository, RepositoryError, StorageConnectionManager, TransactionError,
};

mod validate;

use validate::validate;

pub fn delete_outbound_shipment_service_line(
    connection_manager: &StorageConnectionManager,
    input: DeleteOutboundShipmentLine,
) -> Result<String, DeleteOutboundShipmentServiceLineError> {
    let connection = connection_manager.connection()?;

    let line = connection
        .transaction_sync(|connection| {
            let line = validate(&input, &connection)?;
            InvoiceLineRowRepository::new(&connection).delete(&line.id)?;
            Ok(line)
        })
        .map_err(
            |error: TransactionError<DeleteOutboundShipmentServiceLineError>| match error {
                TransactionError::Transaction { msg, level } => {
                    RepositoryError::TransactionError { msg, level }.into()
                }
                TransactionError::Inner(error) => error,
            },
        )?;
    Ok(line.id)
}

pub enum DeleteOutboundShipmentServiceLineError {
    LineDoesNotExist,
    DatabaseError(RepositoryError),
    InvoiceDoesNotExist,
    NotAnOutboundShipment,
    NotThisStoreInvoice,
    ItemNotFound,
    CannotEditFinalised,
    NotThisInvoiceLine(String),
    NotAServiceItem,
}

impl From<RepositoryError> for DeleteOutboundShipmentServiceLineError {
    fn from(error: RepositoryError) -> Self {
        DeleteOutboundShipmentServiceLineError::DatabaseError(error)
    }
}

impl<ERR> From<WithDBError<ERR>> for DeleteOutboundShipmentServiceLineError
where
    ERR: Into<DeleteOutboundShipmentServiceLineError>,
{
    fn from(result: WithDBError<ERR>) -> Self {
        match result {
            WithDBError::DatabaseError(error) => error.into(),
            WithDBError::Error(error) => error.into(),
        }
    }
}
