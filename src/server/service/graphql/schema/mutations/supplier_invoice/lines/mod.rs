pub mod insert;
pub use self::insert::*;

pub mod upsert;
pub use self::upsert::*;

use crate::business::RequiredInsertField;
use crate::server::service::graphql::schema::mutations::{
    DBError, ForeignKeyError, RecordAlreadyExist, ValueOutOfRange,
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

#[derive(InputObject)]
pub struct UpsertSupplierInvoiceLineInput {
    pub id: String,
    pub item_id: Option<String>,
    pub pack_size: Option<u32>,
    pub batch: Option<String>,
    pub cost_price_per_pack: Option<f64>,
    pub sell_price_per_pack: Option<f64>,
    pub expiry_date: Option<NaiveDate>,
    pub number_of_packs: Option<u32>,
}

#[derive(SimpleObject)]
pub struct InsertSupplierInvoiceLineErrors {
    pub id: String,
    pub errors: Vec<InsertSupplierInvoiceLineError>,
}

#[derive(SimpleObject)]
pub struct UpsertSupplierInvoiceLineErrors {
    pub id: String,
    pub errors: Vec<UpsertSupplierInvoiceLineError>,
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
pub enum UpsertSupplierInvoiceLineError {
    RecordAlreadyExist(RecordAlreadyExist),
    ValueOutOfRange(ValueOutOfRange),
    ForeignKeyError(ForeignKeyError),
    InsertFieldMissing(InsertFieldMissing),
    InvoiceLineIsReserved(InvoiceLineIsReserved),
    InvoiceLineBelongsToAnotherInvoice(InvoiceLineBelongsToAnotherInvoice),
    DBError(DBError),
}

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
pub enum MissingInsertField {
    PackSize,
    NumberOfPacks,
    ItemId,
    CostPricePerPack,
    SellPricePerPack,
}

pub struct InvoiceLineIsReserved;
#[Object]
impl InvoiceLineIsReserved {
    pub async fn description(&self) -> &'static str {
        "Invoice line is reserved"
    }
}

pub struct InvoiceLineBelongsToAnotherInvoice;
#[Object]
impl InvoiceLineBelongsToAnotherInvoice {
    pub async fn description(&self) -> &'static str {
        "Invoice line belongs to another invoice"
    }
}

pub struct InsertFieldMissing(pub MissingInsertField);
#[Object]
impl InsertFieldMissing {
    pub async fn description(&self) -> &'static str {
        "Field missing for insert line"
    }

    pub async fn field(&self) -> &MissingInsertField {
        &self.0
    }
}

impl From<RequiredInsertField> for MissingInsertField {
    fn from(missing_field: RequiredInsertField) -> Self {
        use self::MissingInsertField::*;
        match missing_field {
            RequiredInsertField::PackSize => PackSize,
            RequiredInsertField::NumberOfPacks => NumberOfPacks,
            RequiredInsertField::ItemId => ItemId,
            RequiredInsertField::CostPricePerPack => CostPricePerPack,
            RequiredInsertField::SellPricePerPack => SellPricePerPack,
        }
    }
}
