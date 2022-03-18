use repository::{
    EqualFilter, Name, NameFilter, NameQueryRepository, RepositoryError, StorageConnection,
    StoreRowRepository,
};

pub fn check_store_id_matches(store_id_a: &str, store_id_b: &str) -> bool {
    store_id_a == store_id_b
}

pub fn check_store_exists(
    connection: &StorageConnection,
    store_id: &str,
) -> Result<bool, RepositoryError> {
    Ok(StoreRowRepository::new(&connection)
        .find_one_by_id(store_id)?
        .is_some())
}

pub enum OtherPartyErrors {
    OtherPartyDoesNotExist,
    OtherPartyNotVisible,
    TypeMismatched,
    DatabaseError(RepositoryError),
}

pub enum CheckOtherPartyType {
    Customer,
    Supplier,
}

pub fn get_other_party(
    connection: &StorageConnection,
    store_id: &str,
    other_party_id: &str,
) -> Result<Option<Name>, RepositoryError> {
    NameQueryRepository::new(connection).query_one(
        store_id,
        NameFilter::new().id(EqualFilter::equal_to(other_party_id)),
    )
}

pub fn check_other_party(
    connection: &StorageConnection,
    store_id: &str,
    other_party_id: &str,
    other_party_type: CheckOtherPartyType,
) -> Result<Name, OtherPartyErrors> {
    let other_party = get_other_party(connection, store_id, other_party_id)?
        .ok_or(OtherPartyErrors::OtherPartyDoesNotExist)?;

    if !other_party.is_visible() {
        return Err(OtherPartyErrors::OtherPartyNotVisible);
    }

    match other_party_type {
        CheckOtherPartyType::Customer => {
            if !other_party.is_customer() {
                return Err(OtherPartyErrors::TypeMismatched);
            }
        }

        CheckOtherPartyType::Supplier => {
            if !other_party.is_supplier() {
                return Err(OtherPartyErrors::TypeMismatched);
            }
        }
    };

    Ok(other_party)
}

impl From<RepositoryError> for OtherPartyErrors {
    fn from(error: RepositoryError) -> Self {
        Self::DatabaseError(error)
    }
}
