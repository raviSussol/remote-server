use super::GenericError;
use super::{DBError, ForeignKeyError};
use crate::server::service::graphql::schema::types::{InvoiceStatus, NameQuery};
use async_graphql::*;

pub mod insert;
pub use self::insert::*;

pub mod update;
pub use self::update::*;

#[derive(InputObject)]
pub struct InsertSupplierInvoiceInput {
    pub id: String,
    pub other_party_id: String,
    pub status: InvoiceStatus,
    pub comment: Option<String>,
    pub their_reference: Option<String>,
    // lines
}

#[derive(InputObject)]
pub struct UpdateSupplierInvoiceInput {
    pub id: String,
    pub other_party_id: Option<String>,
    pub status: Option<InvoiceStatus>,
    pub comment: Option<String>,
    pub their_reference: Option<String>,
    // lines
}

#[derive(SimpleObject)]
pub struct InsertSupplierInvoiceErrors {
    id: String,
    errors: Vec<InsertSupplierInvoiceError>,
}

impl InsertSupplierInvoiceErrors {
    fn new(id: String, error: InsertSupplierInvoiceError) -> InsertSupplierInvoiceErrors {
        InsertSupplierInvoiceErrors {
            id,
            errors: vec![error],
        }
    }
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "String"))]
pub enum InsertSupplierInvoiceError {
    ForeignKeyError(ForeignKeyError),
    GenericError(GenericError),
    OtherPartyNotASuppier(OtherPartyNotASuppier),
    DBError(DBError),
}

#[derive(SimpleObject)]
pub struct OtherPartyNotASuppier {
    pub other_party: NameQuery,
    pub description: String,
}

#[derive(SimpleObject)]
pub struct UpdateSupplierInvoiceErrors {
    id: String,
    errors: Vec<UpdateSupplierInvoiceError>,
}

impl UpdateSupplierInvoiceErrors {
    fn new(id: String, error: UpdateSupplierInvoiceError) -> UpdateSupplierInvoiceErrors {
        UpdateSupplierInvoiceErrors {
            id,
            errors: vec![error],
        }
    }
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "String"))]
pub enum UpdateSupplierInvoiceError {
    ForeignKeyError(ForeignKeyError),
    GenericError(GenericError),
    OtherPartyNotASuppier(OtherPartyNotASuppier),
    DBError(DBError),
}
