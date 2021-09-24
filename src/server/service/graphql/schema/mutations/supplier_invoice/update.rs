use super::{
    CannotChangeInvoiceBackToDraft, CannotEditFinalisedInvoice, InvoiceDoesNotBelongToCurrentStore,
    MutationErrorsWrapper, MutationErrorsWrapperNew, NotASupplierInvoice, OtherPartyNotASuppier,
    UpdateSupplierInvoiceError as ApiError,
};
use crate::{
    business::supplier_invoice::UpdateSupplierInvoiceError as BusinessError,
    database::repository::InvoiceRepository,
    server::service::graphql::schema::{
        mutations::{DBError, ForeignKeyError, ForeignKeys, RecordDoesNotExist},
        types::Invoice,
    },
};

use async_graphql::*;

type ApiErrors = MutationErrorsWrapper<ApiError>;

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
                    key: ForeignKeys::OtherPartyId,
                    id: other_party_id,
                })
            }
            BusinessError::OtherPartyIsNotASupplier(name_query) => {
                ApiError::OtherPartyNotASuppier(OtherPartyNotASuppier(name_query))
            }
            BusinessError::InvoiceDoesNotExist => {
                ApiError::RecordDoesNotExist(RecordDoesNotExist {})
            }

            BusinessError::CannotEditFinalisedInvoice => {
                ApiError::CannotEditFinalisedInvoice(CannotEditFinalisedInvoice {})
            }
            BusinessError::NotASupplierInvoice => {
                ApiError::NotASupplierInvoice(NotASupplierInvoice {})
            }
            BusinessError::InvoiceDoesNotBelongToCurrentStore => {
                ApiError::InvoiceDoesNotBelongToCurrentStore(InvoiceDoesNotBelongToCurrentStore {})
            }
            BusinessError::CannoChangeInvoiceBackToDraft => {
                ApiError::CannotChangeInvoiceBackToDraft(CannotChangeInvoiceBackToDraft {})
            }
            BusinessError::DBError(error) => ApiError::DBError(DBError(error)),
        }
    }
}
