use crate::service::{
    invoice::{
        check_invoice_exists, check_invoice_finalised, check_invoice_type, InvoiceDoesNotExist,
        InvoiceIsFinalised, WrongInvoiceType,
    },
    invoice_line::{
        check_batch_exists, check_batch_on_hold, check_item_matches_batch, check_unique_stock_line,
        validate::{
            check_item, check_line_does_not_exists, check_number_of_packs, ItemNotFound,
            LineAlreadyExists, NumberOfPacksBelowOne,
        },
        BatchIsOnHold, ItemDoesNotMatchStockLine, StockLineAlreadyExistsInInvoice,
        StockLineNotFound,
    },
    u32_to_i32,
};
use domain::{invoice::InvoiceType, outbound_shipment::InsertOutboundShipmentLine};
use repository::{
    repository::StorageConnection,
    schema::{InvoiceRow, ItemRow, StockLineRow},
};

use super::InsertOutboundShipmentLineError;

pub fn validate(
    input: &InsertOutboundShipmentLine,
    connection: &StorageConnection,
) -> Result<(ItemRow, InvoiceRow, StockLineRow), InsertOutboundShipmentLineError> {
    check_line_does_not_exists(&input.id, connection)?;
    check_number_of_packs(Some(input.number_of_packs))?;
    let batch = check_batch_exists(&input.stock_line_id, connection)?;
    let item = check_item(&input.item_id, connection)?;
    check_item_matches_batch(&batch, &item)?;
    let invoice = check_invoice_exists(&input.invoice_id, connection)?;
    check_unique_stock_line(
        &input.id,
        &invoice.id,
        Some(input.stock_line_id.to_string()),
        connection,
    )?;
    // check_store(invoice, connection)?; InvoiceDoesNotBelongToCurrentStore
    check_invoice_type(&invoice, InvoiceType::OutboundShipment)?;
    check_invoice_finalised(&invoice)?;

    check_batch_on_hold(&batch)?;
    check_reduction_below_zero(&input, &batch)?;

    Ok((item, invoice, batch))
}

fn check_reduction_below_zero(
    input: &InsertOutboundShipmentLine,
    batch: &StockLineRow,
) -> Result<(), InsertOutboundShipmentLineError> {
    if batch.available_number_of_packs < u32_to_i32(input.number_of_packs) {
        Err(InsertOutboundShipmentLineError::ReductionBelowZero {
            stock_line_id: batch.id.clone(),
        })
    } else {
        Ok(())
    }
}

impl From<BatchIsOnHold> for InsertOutboundShipmentLineError {
    fn from(_: BatchIsOnHold) -> Self {
        InsertOutboundShipmentLineError::BatchIsOnHold
    }
}

impl From<StockLineAlreadyExistsInInvoice> for InsertOutboundShipmentLineError {
    fn from(error: StockLineAlreadyExistsInInvoice) -> Self {
        InsertOutboundShipmentLineError::StockLineAlreadyExistsInInvoice(error.0)
    }
}

impl From<ItemDoesNotMatchStockLine> for InsertOutboundShipmentLineError {
    fn from(_: ItemDoesNotMatchStockLine) -> Self {
        InsertOutboundShipmentLineError::ItemDoesNotMatchStockLine
    }
}

impl From<ItemNotFound> for InsertOutboundShipmentLineError {
    fn from(_: ItemNotFound) -> Self {
        InsertOutboundShipmentLineError::ItemNotFound
    }
}

impl From<StockLineNotFound> for InsertOutboundShipmentLineError {
    fn from(_: StockLineNotFound) -> Self {
        InsertOutboundShipmentLineError::StockLineNotFound
    }
}

impl From<NumberOfPacksBelowOne> for InsertOutboundShipmentLineError {
    fn from(_: NumberOfPacksBelowOne) -> Self {
        InsertOutboundShipmentLineError::NumberOfPacksBelowOne
    }
}

impl From<LineAlreadyExists> for InsertOutboundShipmentLineError {
    fn from(_: LineAlreadyExists) -> Self {
        InsertOutboundShipmentLineError::LineAlreadyExists
    }
}

impl From<WrongInvoiceType> for InsertOutboundShipmentLineError {
    fn from(_: WrongInvoiceType) -> Self {
        InsertOutboundShipmentLineError::NotAnOutboundShipment
    }
}

impl From<InvoiceIsFinalised> for InsertOutboundShipmentLineError {
    fn from(_: InvoiceIsFinalised) -> Self {
        InsertOutboundShipmentLineError::CannotEditFinalised
    }
}

impl From<InvoiceDoesNotExist> for InsertOutboundShipmentLineError {
    fn from(_: InvoiceDoesNotExist) -> Self {
        InsertOutboundShipmentLineError::InvoiceDoesNotExist
    }
}
