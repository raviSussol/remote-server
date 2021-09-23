use super::InsertSupplierInvoiceError;
use crate::database::repository::{NameQueryRepository, RepositoryError};

pub async fn check_other_party(
    name_query_repository: &NameQueryRepository,
    other_party_id: &str,
) -> Result<(), InsertSupplierInvoiceError> {
    use self::InsertSupplierInvoiceError::*;

    let name_query = name_query_repository
        .one(other_party_id)
        .map_err(|error| match &error {
            RepositoryError::NotFound => OtherPartyNotFound(other_party_id.to_string()),
            _ => DBError(error),
        })?;

    match name_query.is_supplier {
        true => Ok(()),
        false => Err(OtherPartyIsNotASupplier(name_query)),
    }
}
