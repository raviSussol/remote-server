use crate::database::repository::{RepositoryError, StoreRepository};

pub async fn current_store_id(
    name_query_repository: &StoreRepository,
) -> Result<String, RepositoryError> {
    // Need to check session for store
    Ok(name_query_repository.all().await?[0].id.clone())
}
