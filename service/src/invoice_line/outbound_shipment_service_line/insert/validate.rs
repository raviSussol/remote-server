use repository::{
    schema::{InvoiceRow, InvoiceRowType, ItemRow, ItemRowType},
    ItemFilter, ItemQueryRepository, RepositoryError, SimpleStringFilter, StorageConnection,
};
use util::constants::DEFAULT_SERVICE_ITEM_CODE;

use crate::{
    invoice::{
        check_invoice_exists, check_invoice_is_editable, check_invoice_type, InvoiceDoesNotExist,
        InvoiceIsNotEditable, WrongInvoiceRowType,
    },
    invoice_line::validate::{
        check_item, check_line_does_not_exists, ItemNotFound, LineAlreadyExists,
    },
};

use super::{InsertOutboundShipmentServiceLine, InsertOutboundShipmentServiceLineError};

type OutError = InsertOutboundShipmentServiceLineError;

pub fn validate(
    input: &InsertOutboundShipmentServiceLine,
    connection: &StorageConnection,
) -> Result<(ItemRow, InvoiceRow), OutError> {
    check_line_does_not_exists(&input.id, connection)?;

    let item = match &input.item_id {
        None => {
            get_default_service_item(connection)?.ok_or(OutError::CannotFindDefaultServiceItem)?
        }
        Some(item_id) => {
            let item = check_item(item_id, connection)?;
            if item.r#type != ItemRowType::Service {
                return Err(OutError::NotAServiceItem);
            }
            item
        }
    };

    let invoice = check_invoice_exists(&input.invoice_id, connection)?;
    // TODO:
    // check_store(invoice, connection)?; InvoiceDoesNotBelongToCurrentStore
    check_invoice_type(&invoice, InvoiceRowType::OutboundShipment)?;
    check_invoice_is_editable(&invoice)?;

    Ok((item, invoice))
}

fn get_default_service_item(
    connection: &StorageConnection,
) -> Result<Option<ItemRow>, RepositoryError> {
    let item_row = ItemQueryRepository::new(connection)
        .query_one(ItemFilter::new().code(SimpleStringFilter::equal_to(DEFAULT_SERVICE_ITEM_CODE)))?
        .map(|item| item.item_row);

    Ok(item_row)
}

impl From<LineAlreadyExists> for OutError {
    fn from(_: LineAlreadyExists) -> Self {
        OutError::LineAlreadyExists
    }
}

impl From<ItemNotFound> for OutError {
    fn from(_: ItemNotFound) -> Self {
        OutError::ItemNotFound
    }
}

impl From<InvoiceDoesNotExist> for OutError {
    fn from(_: InvoiceDoesNotExist) -> Self {
        OutError::InvoiceDoesNotExist
    }
}

impl From<WrongInvoiceRowType> for OutError {
    fn from(_: WrongInvoiceRowType) -> Self {
        OutError::NotAnOutboundShipment
    }
}

impl From<InvoiceIsNotEditable> for OutError {
    fn from(_: InvoiceIsNotEditable) -> Self {
        OutError::CannotEditInvoice
    }
}
