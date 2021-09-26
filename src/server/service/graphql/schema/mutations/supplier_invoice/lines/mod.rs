pub mod insert;
pub use self::insert::*;

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
