use crate::{
    invoice_line::{
        outbound_shipment_line::{
            insert_outbound_shipment_line, update_outbound_shipment_line,
            InsertOutboundShipmentLine, InsertOutboundShipmentLineError,
            UpdateOutboundShipmentLine, UpdateOutboundShipmentLineError,
        },
        validate::check_line_exists_option,
    },
    service_provider::ServiceContext,
};
use repository::{
    schema::{InvoiceLineRow, InvoiceLineRowType},
    InvoiceLine, RepositoryError, StorageConnection,
};

use super::{
    delete_outbound_shipment_unallocated_line, update_outbound_shipment_unallocated_line,
    DeleteOutboundShipmentUnallocatedLine, DeleteOutboundShipmentUnallocatedLineError,
    UpdateOutboundShipmentUnallocatedLine, UpdateOutboundShipmentUnallocatedLineError,
};

mod generate;
mod test;
use generate::{generate, GenerateOutput};

#[derive(Clone, Debug, PartialEq)]
pub struct InputWithError<I, E>
where
    I: Clone + std::fmt::Debug + PartialEq,
    E: Clone + std::fmt::Debug + PartialEq,
{
    input: I,
    error: E,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AllocateOutboundShipmentUnallocatedLineError {
    LineDoesNotExist,
    LineIsNotUnallocatedLine,
    // TODO NotThisStoreInvoice,
    // Internal
    InsertOutboundShipmentLine(
        InputWithError<InsertOutboundShipmentLine, InsertOutboundShipmentLineError>,
    ),
    UpdateOutboundShipmentLine(
        InputWithError<UpdateOutboundShipmentLine, UpdateOutboundShipmentLineError>,
    ),
    DeleteOutboundShipmentUnallocatedLine(
        InputWithError<
            DeleteOutboundShipmentUnallocatedLine,
            DeleteOutboundShipmentUnallocatedLineError,
        >,
    ),
    UpdateOutboundShipmentUnallocatedLine(
        InputWithError<
            UpdateOutboundShipmentUnallocatedLine,
            UpdateOutboundShipmentUnallocatedLineError,
        >,
    ),
    DatabaseError(RepositoryError),
}

type OutError = AllocateOutboundShipmentUnallocatedLineError;

#[derive(Default, Debug, PartialEq)]
pub struct Return {
    inserts: Vec<InvoiceLine>,
    deletes: Vec<String>,
    updates: Vec<InvoiceLine>,
}

pub type AllocateOutboundShipmentUnallocatedLineResult = Result<Return, OutError>;

pub fn allocate_outbound_shipment_unallocated_line(
    ctx: &ServiceContext,
    store_id: &str,
    line_id: String,
) -> AllocateOutboundShipmentUnallocatedLineResult {
    let line = ctx
        .connection
        .transaction_sync(|connection| {
            let unallocated_line = validate(connection, &line_id)?;
            let GenerateOutput {
                update_lines,
                insert_lines,
                update_unallocated_line,
                delete_unallocated_line,
            } = generate(&connection, &store_id, unallocated_line)?;

            let mut result = Return::default();

            for input in update_lines.into_iter() {
                result.updates.push(
                    update_outbound_shipment_line(ctx, store_id, input.clone()).map_err(
                        |error| {
                            OutError::UpdateOutboundShipmentLine(InputWithError { input, error })
                        },
                    )?,
                );
            }

            for input in insert_lines.into_iter() {
                result.inserts.push(
                    insert_outbound_shipment_line(ctx, store_id, input.clone()).map_err(
                        |error| {
                            OutError::InsertOutboundShipmentLine(InputWithError { input, error })
                        },
                    )?,
                );
            }

            if let Some(input) = update_unallocated_line {
                result.updates.push(
                    update_outbound_shipment_unallocated_line(ctx, store_id, input.clone())
                        .map_err(|error| {
                            OutError::UpdateOutboundShipmentUnallocatedLine(InputWithError {
                                input,
                                error,
                            })
                        })?,
                );
            }

            if let Some(input) = delete_unallocated_line {
                result.deletes.push(
                    delete_outbound_shipment_unallocated_line(ctx, store_id, input.clone())
                        .map_err(|error| {
                            OutError::DeleteOutboundShipmentUnallocatedLine(InputWithError {
                                input,
                                error,
                            })
                        })?,
                );
            }

            Ok(result) as AllocateOutboundShipmentUnallocatedLineResult
        })
        .map_err(|error| error.to_inner_error())?;
    Ok(line)
}

fn validate(connection: &StorageConnection, line_id: &str) -> Result<InvoiceLineRow, OutError> {
    let invoice_line =
        check_line_exists_option(connection, line_id)?.ok_or(OutError::LineDoesNotExist)?;

    if invoice_line.r#type != InvoiceLineRowType::UnallocatedStock {
        return Err(OutError::LineIsNotUnallocatedLine);
    }

    Ok(invoice_line)
}

impl From<RepositoryError> for AllocateOutboundShipmentUnallocatedLineError {
    fn from(error: RepositoryError) -> Self {
        AllocateOutboundShipmentUnallocatedLineError::DatabaseError(error)
    }
}
