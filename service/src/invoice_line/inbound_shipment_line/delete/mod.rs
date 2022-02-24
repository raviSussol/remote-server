use crate::WithDBError;
use repository::{
    InvoiceLineRowRepository, RepositoryError, StockLineRowRepository, StorageConnectionManager,
    TransactionError,
};

mod validate;

use validate::validate;
pub struct DeleteInboundShipmentLine {
    pub id: String,
    pub invoice_id: String,
}

pub fn delete_inbound_shipment_line(
    connection_manager: &StorageConnectionManager,
    input: DeleteInboundShipmentLine,
) -> Result<String, DeleteInboundShipmentLineError> {
    let connection = connection_manager.connection()?;
    let line = connection
        .transaction_sync(|connection| {
            let line = validate(&input, &connection)?;

            let delete_batch_id_option = line.stock_line_id.clone();

            InvoiceLineRowRepository::new(&connection).delete(&line.id)?;

            if let Some(id) = delete_batch_id_option {
                StockLineRowRepository::new(&connection).delete(&id)?;
            }
            Ok(line)
        })
        .map_err(
            |error: TransactionError<DeleteInboundShipmentLineError>| match error {
                TransactionError::Transaction { msg, level } => {
                    RepositoryError::TransactionError { msg, level }.into()
                }
                TransactionError::Inner(error) => error,
            },
        )?;
    Ok(line.id)
}
pub enum DeleteInboundShipmentLineError {
    LineDoesNotExist,
    DatabaseError(RepositoryError),
    InvoiceDoesNotExist,
    NotAnInboundShipment,
    NotThisStoreInvoice,
    CannotEditFinalised,
    BatchIsReserved,
    NotThisInvoiceLine(String),
}

impl From<RepositoryError> for DeleteInboundShipmentLineError {
    fn from(error: RepositoryError) -> Self {
        DeleteInboundShipmentLineError::DatabaseError(error)
    }
}

impl<ERR> From<WithDBError<ERR>> for DeleteInboundShipmentLineError
where
    ERR: Into<DeleteInboundShipmentLineError>,
{
    fn from(result: WithDBError<ERR>) -> Self {
        match result {
            WithDBError::DatabaseError(error) => error.into(),
            WithDBError::Error(error) => error.into(),
        }
    }
}
