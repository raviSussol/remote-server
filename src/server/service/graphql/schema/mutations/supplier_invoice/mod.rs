use super::RecordAlreadyExist;
use super::{DBError, ForeignKeyError, RecordDoesNotExist};
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
#[graphql(concrete(
    name = "InsertSupplierInvoiceErrors",
    params(InsertSupplierInvoiceError)
))]
#[graphql(concrete(
    name = "UpdateSupplierInvoiceErrors",
    params(UpdateSupplierInvoiceError)
))]
pub struct MutationErrorsWrapper<ErrorType: OutputType> {
    id: String,
    errors: Vec<ErrorType>,
}

pub trait MutationErrorsWrapperNew<MutationErrorsWrapperType, ErrorType: OutputType> {
    fn new(id: String, error: ErrorType) -> MutationErrorsWrapperType;
}

impl<ErrorType: OutputType> MutationErrorsWrapperNew<MutationErrorsWrapper<ErrorType>, ErrorType>
    for MutationErrorsWrapper<ErrorType>
{
    fn new(id: String, error: ErrorType) -> MutationErrorsWrapper<ErrorType> {
        MutationErrorsWrapper {
            id,
            errors: vec![error],
        }
    }
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "String"))]
pub enum InsertSupplierInvoiceError {
    ForeignKeyError(ForeignKeyError),
    RecordAlreadyExist(RecordAlreadyExist),
    OtherPartyNotASuppier(OtherPartyNotASuppier),
    DBError(DBError),
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "String"))]
pub enum UpdateSupplierInvoiceError {
    ForeignKeyError(ForeignKeyError),
    RecordDoesNotExist(RecordDoesNotExist),
    NotASupplierInvoice(NotASupplierInvoice),
    OtherPartyNotASuppier(OtherPartyNotASuppier),
    CannotEditFinalisedInvoice(CannotEditFinalisedInvoice),
    InvoiceDoesNotBelongToCurrentStore(InvoiceDoesNotBelongToCurrentStore),
    CannotChangeInvoiceBackToDraft(CannotChangeInvoiceBackToDraft),
    DBError(DBError),
}

pub struct OtherPartyNotASuppier(NameQuery);
#[Object]
impl OtherPartyNotASuppier {
    pub async fn description(&self) -> &'static str {
        "Other party name is not a supplier"
    }

    pub async fn other_party(&self) -> &NameQuery {
        &self.0
    }
}

pub struct CannotEditFinalisedInvoice;
#[Object]
impl CannotEditFinalisedInvoice {
    pub async fn description(&self) -> &'static str {
        "Cannot edit finalised invoice"
    }
}

pub struct NotASupplierInvoice;
#[Object]
impl NotASupplierInvoice {
    pub async fn description(&self) -> &'static str {
        "Invoice is not Supplier Invoice"
    }
}

pub struct InvoiceDoesNotBelongToCurrentStore;
#[Object]
impl InvoiceDoesNotBelongToCurrentStore {
    pub async fn description(&self) -> &'static str {
        "Invoice does not belong to current store"
    }
}

pub struct CannotChangeInvoiceBackToDraft;
#[Object]
impl CannotChangeInvoiceBackToDraft {
    pub async fn description(&self) -> &'static str {
        "Cannot change invoice back to draft"
    }
}
