use super::{
    InsertSupplierInvoiceError as ApiError, InsertSupplierInvoiceErrors as ApiErrors,
    OtherPartyNotASuppier,
};
use crate::{
    business::supplier_invoice::InsertSupplierInvoiceError as BusinessError,
    database::repository::InvoiceRepository,
    server::service::graphql::schema::{
        mutations::{DBError, ForeignKeyError, ForeignKeys, GenericError},
        types::Invoice,
    },
};

use async_graphql::*;

#[derive(Union)]
pub enum InvoiceOrInsertSupplierInvoiceError {
    Invoice(Invoice),
    Errors(ApiErrors),
}
use self::InvoiceOrInsertSupplierInvoiceError as InvoiceWithError;

impl InvoiceOrInsertSupplierInvoiceError {
    pub async fn new(
        id: String,
        insert_result: Result<(), BusinessError>,
        invoice_repository: &InvoiceRepository,
    ) -> InvoiceWithError {
        match insert_result {
            Ok(_) => invoice_result(id, invoice_repository).await,
            Err(error) => error_result(id, error),
        }
    }
}

async fn invoice_result(id: String, invoice_repository: &InvoiceRepository) -> InvoiceWithError {
    match invoice_repository.find_one_by_id(&id).await {
        Ok(invoice_row) => InvoiceWithError::Invoice(Invoice { invoice_row }),
        Err(error) => {
            InvoiceWithError::Errors(ApiErrors::new(id, ApiError::DBError(DBError(error))))
        }
    }
}

fn error_result(id: String, error: BusinessError) -> InvoiceWithError {
    InvoiceWithError::Errors(ApiErrors::new(id, error.into()))
}

impl From<BusinessError> for ApiError {
    fn from(business_error: BusinessError) -> Self {
        match business_error {
            BusinessError::OtherPartyNotFound(other_party_id) => {
                ApiError::ForeignKeyError(ForeignKeyError {
                    description: "Name with other party id does not exist".to_string(),
                    key: ForeignKeys::OtherPartyId,
                    key_id: other_party_id,
                })
            }
            BusinessError::OtherPartyIsNotASupplier(name_query) => {
                ApiError::OtherPartyNotASuppier(OtherPartyNotASuppier {
                    description: "Other party name is not a supplier".to_string(),
                    other_party: name_query,
                })
            }
            BusinessError::InvoiceExists => ApiError::GenericError(GenericError {
                description: "Invoice with this id already exists".to_string(),
            }),
            BusinessError::DBError(error) => ApiError::DBError(DBError(error)),
        }
    }
}
