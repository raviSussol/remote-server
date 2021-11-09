use async_graphql::*;

use crate::{
    server::service::graphql::schema::{
        mutations::{
            CannotEditFinalisedInvoice, ForeignKey, ForeignKeyError,
            InvoiceDoesNotBelongToCurrentStore, InvoiceLineBelongsToAnotherInvoice,
            NotAnOutboundShipment,
        },
        types::{
            get_invoice_line_response, DatabaseError, ErrorWrapper, InvoiceLineResponse, Range,
            RangeError, RangeField, RecordNotFound,
        },
    },
    service::invoice_line::{update_outbound_shipment_line, UpdateOutboundShipmentLineError},
};
use domain::outbound_shipment::UpdateOutboundShipmentLine;
use repository::repository::StorageConnectionManager;

use super::{
    ItemDoesNotMatchStockLine, LineDoesNotReferenceStockLine, NotEnoughStockForReduction,
    StockLineAlreadyExistsInInvoice, StockLineDoesNotBelongToCurrentStore, StockLineIsOnHold,
};

#[derive(InputObject)]
pub struct UpdateOutboundShipmentLineInput {
    pub id: String,
    invoice_id: String,
    item_id: Option<String>,
    stock_line_id: Option<String>,
    number_of_packs: Option<u32>,
}

pub fn get_update_outbound_shipment_line_response(
    connection_manager: &StorageConnectionManager,
    input: UpdateOutboundShipmentLineInput,
) -> UpdateOutboundShipmentLineResponse {
    use UpdateOutboundShipmentLineResponse::*;
    match update_outbound_shipment_line(connection_manager, input.into()) {
        Ok(id) => Response(get_invoice_line_response(connection_manager, id)),
        Err(error) => error.into(),
    }
}

#[derive(Union)]
pub enum UpdateOutboundShipmentLineResponse {
    Error(ErrorWrapper<UpdateOutboundShipmentLineErrorInterface>),
    #[graphql(flatten)]
    Response(InvoiceLineResponse),
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum UpdateOutboundShipmentLineErrorInterface {
    DatabaseError(DatabaseError),
    ForeignKeyError(ForeignKeyError),
    RecordNotFound(RecordNotFound),
    CannotEditFinalisedInvoice(CannotEditFinalisedInvoice),
    InvoiceDoesNotBelongToCurrentStore(InvoiceDoesNotBelongToCurrentStore),
    StockLineDoesNotBelongToCurrentStore(StockLineDoesNotBelongToCurrentStore),
    LineDoesNotReferenceStockLine(LineDoesNotReferenceStockLine),
    ItemDoesNotMatchStockLine(ItemDoesNotMatchStockLine),
    StockLineAlreadyExistsInInvoice(StockLineAlreadyExistsInInvoice),
    InvoiceLineBelongsToAnotherInvoice(InvoiceLineBelongsToAnotherInvoice),
    NotAnOutboundShipment(NotAnOutboundShipment),
    RangeError(RangeError),
    StockLineIsOnHold(StockLineIsOnHold),
    NotEnoughStockForReduction(NotEnoughStockForReduction),
}

impl From<UpdateOutboundShipmentLineInput> for UpdateOutboundShipmentLine {
    fn from(
        UpdateOutboundShipmentLineInput {
            id,
            invoice_id,
            item_id,
            stock_line_id,
            number_of_packs,
        }: UpdateOutboundShipmentLineInput,
    ) -> Self {
        UpdateOutboundShipmentLine {
            id,
            invoice_id,
            item_id,
            stock_line_id,
            number_of_packs,
        }
    }
}

impl From<UpdateOutboundShipmentLineError> for UpdateOutboundShipmentLineResponse {
    fn from(error: UpdateOutboundShipmentLineError) -> Self {
        use UpdateOutboundShipmentLineErrorInterface as OutError;
        let error = match error {
            UpdateOutboundShipmentLineError::DatabaseError(error) => {
                OutError::DatabaseError(DatabaseError(error))
            }
            UpdateOutboundShipmentLineError::InvoiceDoesNotExist => {
                OutError::ForeignKeyError(ForeignKeyError(ForeignKey::InvoiceId))
            }
            UpdateOutboundShipmentLineError::NotAnOutboundShipment => {
                OutError::NotAnOutboundShipment(NotAnOutboundShipment {})
            }
            UpdateOutboundShipmentLineError::NotThisStoreInvoice => {
                OutError::InvoiceDoesNotBelongToCurrentStore(InvoiceDoesNotBelongToCurrentStore {})
            }
            UpdateOutboundShipmentLineError::CannotEditFinalised => {
                OutError::CannotEditFinalisedInvoice(CannotEditFinalisedInvoice {})
            }
            UpdateOutboundShipmentLineError::ItemNotFound => {
                OutError::ForeignKeyError(ForeignKeyError(ForeignKey::ItemId))
            }
            UpdateOutboundShipmentLineError::NumberOfPacksBelowOne => {
                OutError::RangeError(RangeError {
                    field: RangeField::NumberOfPacks,
                    range: Range::Min(1),
                })
            }
            UpdateOutboundShipmentLineError::StockLineNotFound => {
                OutError::ForeignKeyError(ForeignKeyError(ForeignKey::StockLineId))
            }
            UpdateOutboundShipmentLineError::StockLineAlreadyExistsInInvoice(line_id) => {
                OutError::StockLineAlreadyExistsInInvoice(StockLineAlreadyExistsInInvoice(line_id))
            }
            UpdateOutboundShipmentLineError::ItemDoesNotMatchStockLine => {
                OutError::ItemDoesNotMatchStockLine(ItemDoesNotMatchStockLine {})
            }
            UpdateOutboundShipmentLineError::LineDoesNotExist => {
                OutError::RecordNotFound(RecordNotFound {})
            }
            UpdateOutboundShipmentLineError::LineDoesNotReferenceStockLine => {
                OutError::LineDoesNotReferenceStockLine(LineDoesNotReferenceStockLine {})
            }
            UpdateOutboundShipmentLineError::ReductionBelowZero {
                stock_line_id,
                line_id,
            } => OutError::NotEnoughStockForReduction(NotEnoughStockForReduction {
                stock_line_id,
                line_id: Some(line_id),
            }),
            UpdateOutboundShipmentLineError::NotThisInvoiceLine(invoice_id) => {
                OutError::InvoiceLineBelongsToAnotherInvoice(InvoiceLineBelongsToAnotherInvoice(
                    invoice_id,
                ))
            }
            UpdateOutboundShipmentLineError::BatchIsOnHold => {
                OutError::StockLineIsOnHold(StockLineIsOnHold {})
            }
        };

        UpdateOutboundShipmentLineResponse::Error(ErrorWrapper { error })
    }
}
