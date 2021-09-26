use super::{
    CannotChangeInvoiceBackToDraft, CannotEditFinalisedInvoice, InvoiceDoesNotBelongToCurrentStore,
    NotASupplierInvoice, OptVec, OtherPartyNotASuppier, UpdateSupplierInvoiceError as ApiError,
    UpdateSupplierInvoiceErrors as ApiErrors, UpsertSupplierInvoiceLineErrors as ApiLineError,
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
        Err(error) => InvoiceWithError::Errors(ApiErrors {
            id,
            errors: Some(vec![ApiError::DBError(DBError(error))]),
            lines: None,
        }),
    }
}

impl From<ApiError> for OptVec<ApiError> {
    fn from(error: ApiError) -> Self {
        Some(vec![error])
    }
}

fn error_result(id: String, error: BusinessError) -> InvoiceWithError {
    let (errors, lines) = error.into();
    InvoiceWithError::Errors(ApiErrors { id, errors, lines })
}

impl From<BusinessError> for (OptVec<ApiError>, OptVec<ApiLineError>) {
    fn from(business_error: BusinessError) -> Self {
        let api_error = match business_error {
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
            BusinessError::InvoiceLineErrors(invoice_line_errors) => {
                return (
                    None,
                    Some(
                        invoice_line_errors
                            .into_iter()
                            .map(ApiLineError::from)
                            .collect(),
                    ),
                )
            }
        };
        (api_error.into(), None)
    }
}
