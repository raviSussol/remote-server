use super::{
    InsertSupplierInvoiceLineError as SingleApiError,
    InsertSupplierInvoiceLineErrors as ApiLineError,
};
use crate::{
    business::{
        InsertSupplierInvoiceLineError as BusinessSingleError,
        InsertSupplierInvoiceLineErrors as BusinessLineError,
    },
    server::service::graphql::schema::mutations::{
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
            BusinessSingleError::InvoiceLineAlreadyExists => {
                SingleApiError::RecordAlreadyExist(RecordAlreadyExist {})
            }
            BusinessSingleError::ItemIdNotFound(item_id) => {
                SingleApiError::ForeignKeyError(ForeignKeyError {
                    key: ForeignKeys::ItemId,
                    id: item_id,
                })
            }
        }
    }
}
