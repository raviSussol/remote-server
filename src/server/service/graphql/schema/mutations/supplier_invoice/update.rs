use super::{
    OtherPartyNotASuppier, UpdateSupplierInvoiceError as ApiError,
    UpdateSupplierInvoiceErrors as ApiErrors,
};
use crate::{
    business::supplier_invoice::UpdateSupplierInvoiceError as BusinessError,
    database::repository::InvoiceRepository,
    server::service::graphql::schema::{
        mutations::{DBError, ForeignKeyError, ForeignKeys, GenericError},
        types::Invoice,
    },
};

use async_graphql::*;

#[derive(Union)]
pub enum InvoiceOrUpdateSupplierInvoiceError {
    Invoice(Invoice),
    Errors(ApiErrors),
}
use self::InvoiceOrUpdateSupplierInvoiceError as InvoiceWithError;

impl InvoiceOrUpdateSupplierInvoiceError {
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
            BusinessError::InvoiceDoesNotExist => ApiError::GenericError(GenericError {
                description: "Invoice with this id does not exist".to_string(),
            }),

            BusinessError::CannotEditFinalisedInvoice => ApiError::GenericError(GenericError {
                description: "Cannot edit finalised invoice".to_string(),
            }),
            BusinessError::NotASupplierInvoice => ApiError::GenericError(GenericError {
                description: "Not a supplier invoice".to_string(),
            }),
            BusinessError::InvoiceDoesNotBelongToCurrentStore => {
                ApiError::GenericError(GenericError {
                    description: "Invoice does not belong to current store".to_string(),
                })
            }
            BusinessError::CannoChangeInvoiceBackToDraft => ApiError::GenericError(GenericError {
                description: "Canno change invoice back to draft".to_string(),
            }),
            BusinessError::DBError(error) => ApiError::DBError(DBError(error)),
        }
    }
}
