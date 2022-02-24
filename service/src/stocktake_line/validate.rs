use domain::EqualFilter;
use repository::{
    schema::StocktakeLineRow, ItemFilter, ItemQueryRepository, LocationRepository, RepositoryError,
    StocktakeLineRowRepository, StorageConnection, LocationFilter,
};

pub fn check_stocktake_line_exist(
    connection: &StorageConnection,
    id: &str,
) -> Result<Option<StocktakeLineRow>, RepositoryError> {
    StocktakeLineRowRepository::new(&connection).find_one_by_id(id)
}

pub fn check_location_exists(
    connection: &StorageConnection,
    id: &str,
) -> Result<bool, RepositoryError> {
    let count = LocationRepository::new(connection)
        .count(Some(LocationFilter::new().id(EqualFilter::equal_to(id))))?;
    Ok(count == 1)
}

pub fn check_item_exists(
    connection: &StorageConnection,
    id: &str,
) -> Result<bool, RepositoryError> {
    let count = ItemQueryRepository::new(connection)
        .count(Some(ItemFilter::new().id(EqualFilter::equal_to(id))))?;
    Ok(count == 1)
}
