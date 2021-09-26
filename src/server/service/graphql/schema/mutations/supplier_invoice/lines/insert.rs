use crate::{
    business::{
        InsertSupplierInvoiceLineError as BusinessSingleError,
        InsertSupplierInvoiceLineErrors as BusinessLineError,
    },
    server::service::graphql::schema::mutations::{
        DBError, ForeignKeyError, ForeignKeys, RangeFields, RecordAlreadyExist, ValueOutOfRange,
    },
};
use async_graphql::*;
use chrono::NaiveDate;

#[derive(InputObject)]
pub struct InsertSupplierInvoiceLineInput {
    pub id: String,
    pub item_id: String,
    pub pack_size: u32,
    pub batch: Option<String>,
    pub cost_price_per_pack: f64,
    pub sell_price_per_pack: f64,
    pub expiry_date: Option<NaiveDate>,
    pub number_of_packs: u32,
}

#[derive(SimpleObject)]
pub struct InsertSupplierInvoiceLineErrors {
    pub id: String,
    pub errors: Vec<InsertSupplierInvoiceLineError>,
}

#[derive(SimpleObject)]
pub struct UpdateSupplierInvoiceLineErrors {
    pub id: String,
    pub errors: Vec<UpdateSupplierInvoiceLineError>,
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum InsertSupplierInvoiceLineError {
    RecordAlreadyExist(RecordAlreadyExist),
    ValueOutOfRange(ValueOutOfRange),
    ForeignKeyError(ForeignKeyError),
    DBError(DBError),
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum UpdateSupplierInvoiceLineError {
    DBError(DBError),
}

type SingleApiError = InsertSupplierInvoiceLineError;
type ApiLineError = InsertSupplierInvoiceLineErrors;

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
