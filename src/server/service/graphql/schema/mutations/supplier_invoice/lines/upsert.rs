use super::{
    InvoiceLineBelongsToAnotherInvoice, UpsertSupplierInvoiceLineError as SingleApiError,
    UpsertSupplierInvoiceLineErrors as ApiLineError,
};
use crate::{
    business::{
        UpsertSupplierInvoiceLineError as BusinessSingleError,
        UpsertSupplierInvoiceLineErrors as BusinessLineError,
    },
    server::service::graphql::schema::mutations::{
        supplier_invoice::{InsertFieldMissing, InvoiceLineIsReserved},
        ForeignKeyError, ForeignKeys, RangeFields, RecordAlreadyExist, ValueOutOfRange,
    },
};

impl From<BusinessLineError> for ApiLineError {
    fn from(error: BusinessLineError) -> Self {
        ApiLineError {
            id: error.id,
            errors: error.errors.into_iter().map(SingleApiError::from).collect(),
        }
    }
}

impl From<BusinessSingleError> for SingleApiError {
    fn from(error: BusinessSingleError) -> Self {
        match error {
            BusinessSingleError::PackSizeMustBeAboveOne(pack_size) => {
                SingleApiError::ValueOutOfRange(ValueOutOfRange {
                    field: RangeFields::PackSize,
                    description: format!("Pack size must be at least one, got {}", pack_size),
                })
            }
            BusinessSingleError::SellPricePerPackMustBePositive(price) => {
                SingleApiError::ValueOutOfRange(ValueOutOfRange {
                    field: RangeFields::SellPricePerPack,
                    description: format!("Sell price must be above zero, got {}", price),
                })
            }
            BusinessSingleError::CostPricePerPackMustBePositive(price) => {
                SingleApiError::ValueOutOfRange(ValueOutOfRange {
                    field: RangeFields::CostPricePerPack,
                    description: format!("Cost price must be above zero, got {}", price),
                })
            }
            BusinessSingleError::ItemIdNotFound(item_id) => {
                SingleApiError::ForeignKeyError(ForeignKeyError {
                    key: ForeignKeys::ItemId,
                    id: item_id,
                })
            }
            BusinessSingleError::InsertFieldMissing(field) => {
                SingleApiError::InsertFieldMissing(InsertFieldMissing(field.into()))
            }
            BusinessSingleError::InvoiceLineIsReserved => {
                SingleApiError::InvoiceLineIsReserved(InvoiceLineIsReserved {})
            }
            BusinessSingleError::InvoiceLineBelongsToAnotherInvoice => {
                SingleApiError::InvoiceLineBelongsToAnotherInvoice(
                    InvoiceLineBelongsToAnotherInvoice {},
                )
            }
        }
    }
}
