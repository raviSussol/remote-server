use super::{InsertSupplierInvoiceError, UpdateSupplierInvoiceError};
use crate::{
    database::repository::{NameQueryRepository, RepositoryError},
    server::service::graphql::schema::types::NameQuery,
};

enum CheckOtherParty {
    DBError(RepositoryError),
    NotASupplier(NameQuery),
    NotFound(String),
}

pub async fn check_other_party_insert(
    name_query_repository: &NameQueryRepository,
    other_party_id: &str,
) -> Result<(), InsertSupplierInvoiceError> {
    Ok(check_other_party(name_query_repository, other_party_id)?)
}

pub async fn check_other_party_update(
    name_query_repository: &NameQueryRepository,
    other_party_id: &Option<String>,
) -> Result<(), UpdateSupplierInvoiceError> {
    match other_party_id {
        Some(other_party_id) => Ok(check_other_party(name_query_repository, &other_party_id)?),
        None => Ok(()),
    }
}

fn check_other_party(
    name_query_repository: &NameQueryRepository,
    other_party_id: &str,
) -> Result<(), CheckOtherParty> {
    use self::CheckOtherParty::*;

    let name_query = name_query_repository
        .one(other_party_id)
        .map_err(|error| match &error {
            RepositoryError::NotFound => NotFound(other_party_id.to_string()),
            _ => DBError(error),
        })?;

    match name_query.is_supplier {
        true => Ok(()),
        false => Err(NotASupplier(name_query)),
    }
}

impl From<CheckOtherParty> for InsertSupplierInvoiceError {
    fn from(error: CheckOtherParty) -> Self {
        use self::InsertSupplierInvoiceError::*;
        match error {
            CheckOtherParty::DBError(error) => DBError(error),
            CheckOtherParty::NotASupplier(name_query) => OtherPartyIsNotASupplier(name_query),
            CheckOtherParty::NotFound(other_party_id) => OtherPartyNotFound(other_party_id),
        }
    }
}

impl From<CheckOtherParty> for UpdateSupplierInvoiceError {
    fn from(error: CheckOtherParty) -> Self {
        use self::UpdateSupplierInvoiceError::*;
        match error {
            CheckOtherParty::DBError(error) => DBError(error),
            CheckOtherParty::NotASupplier(name_query) => OtherPartyIsNotASupplier(name_query),
            CheckOtherParty::NotFound(other_party_id) => OtherPartyNotFound(other_party_id),
        }
    }
}
