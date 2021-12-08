use crate::WithDBError;
use domain::{
    invoice::{InvoiceStatus, InvoiceType},
    invoice_line::{InvoiceLine, InvoiceLineFilter},
    EqualFilter,
};
use repository::{
    schema::{InvoiceRow, InvoiceRowType},
    InvoiceLineRepository, InvoiceRepository, RepositoryError, StorageConnection,
};

pub struct WrongInvoiceType;

pub fn check_invoice_type(
    invoice: &InvoiceRow,
    r#type: InvoiceType,
) -> Result<(), WrongInvoiceType> {
    if invoice.r#type != r#type.into() {
        Err(WrongInvoiceType {})
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub enum InvoiceIsNotEditable {
    InboundShippmentAllocated,
    InboundShippmentPicked,
    InboundShippmentVerified,
    OutboundShippmentShipped,
    OutboundShippmentDelivered,
    OutboundShippmentVerified,
}

pub fn check_invoice_is_editable(invoice: &InvoiceRow) -> Result<(), InvoiceIsNotEditable> {
    use InvoiceIsNotEditable::*;
    let status = InvoiceStatus::from(invoice.status.clone());
    match &invoice.r#type {
        InvoiceRowType::OutboundShipment => match &status {
            InvoiceStatus::New => {}
            InvoiceStatus::Allocated => {}
            InvoiceStatus::Picked => {}
            InvoiceStatus::Shipped => return Err(OutboundShippmentShipped),
            InvoiceStatus::Delivered => return Err(OutboundShippmentDelivered),
            InvoiceStatus::Verified => return Err(OutboundShippmentVerified),
        },
        InvoiceRowType::InboundShipment => match &status {
            InvoiceStatus::New => {}
            InvoiceStatus::Shipped => {}
            InvoiceStatus::Delivered => {}
            InvoiceStatus::Allocated => return Err(InboundShippmentAllocated),
            InvoiceStatus::Picked => return Err(InboundShippmentPicked),
            InvoiceStatus::Verified => return Err(InboundShippmentVerified),
        },
    };

    Ok(())
}
pub enum InvoiceStatusError {
    CannotChangeStatusOfInvoiceOnHold,
    CannotReverseInvoiceStatus,
}

pub fn check_invoice_status(
    invoice: &InvoiceRow,
    status_option: Option<InvoiceStatus>,
    on_hold_option: &Option<bool>,
) -> Result<(), InvoiceStatusError> {
    if let Some(new_status) = status_option {
        let existing_status: InvoiceStatus = invoice.status.clone().into();
        // When we update invoice, error will trigger if
        // * invoice is currently on hold and is not being change to be not on hold
        let is_not_on_hold = !invoice.on_hold || !on_hold_option.unwrap_or(true);

        if new_status != existing_status && !is_not_on_hold {
            return Err(InvoiceStatusError::CannotChangeStatusOfInvoiceOnHold);
        }
        if new_status.index() < existing_status.index() {
            return Err(InvoiceStatusError::CannotReverseInvoiceStatus);
        }
    }

    Ok(())
}

pub struct InvoiceDoesNotExist;

pub fn check_invoice_exists(
    id: &str,
    connection: &StorageConnection,
) -> Result<InvoiceRow, WithDBError<InvoiceDoesNotExist>> {
    let result = InvoiceRepository::new(connection).find_one_by_id(id);

    match result {
        Ok(invoice_row) => Ok(invoice_row),
        Err(RepositoryError::NotFound) => Err(WithDBError::err(InvoiceDoesNotExist)),
        Err(error) => Err(WithDBError::db(error)),
    }
}

pub struct InvoiceLinesExist(pub Vec<InvoiceLine>);

pub fn check_invoice_is_empty(
    id: &str,
    connection: &StorageConnection,
) -> Result<(), WithDBError<InvoiceLinesExist>> {
    let lines = InvoiceLineRepository::new(connection)
        .query_by_filter(InvoiceLineFilter::new().invoice_id(EqualFilter::equal_to(id)))
        .map_err(WithDBError::db)?;

    if lines.len() > 0 {
        Err(WithDBError::err(InvoiceLinesExist(lines)))
    } else {
        Ok(())
    }
}
