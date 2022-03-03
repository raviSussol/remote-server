use async_graphql::dataloader::*;
use repository::EqualFilter;
use std::collections::HashMap;

use repository::{Name, NameFilter};
use repository::{NameQueryRepository, RepositoryError, StorageConnectionManager};

use super::IdAndStoreId;

pub struct NameByIdLoader {
    pub connection_manager: StorageConnectionManager,
}

#[async_trait::async_trait]
impl Loader<IdAndStoreId> for NameByIdLoader {
    type Value = Name;
    type Error = RepositoryError;

    async fn load(
        &self,
        ids_with_store_id: &[IdAndStoreId],
    ) -> Result<HashMap<IdAndStoreId, Self::Value>, Self::Error> {
        let connection = self.connection_manager.connection()?;
        let repo = NameQueryRepository::new(&connection);

        let store_id = match IdAndStoreId::get_store_id(ids_with_store_id) {
            Some(store_id) => store_id,
            None => return Ok(HashMap::new()),
        };

        Ok(repo
            .query_by_filter(
                store_id,
                NameFilter::new().id(EqualFilter::equal_any(IdAndStoreId::get_ids(
                    ids_with_store_id,
                ))),
            )?
            .into_iter()
            .map(|name| {
                (
                    IdAndStoreId {
                        id: name.name_row.id.clone(),
                        store_id: store_id.to_string(),
                    },
                    name,
                )
            })
            .collect())
    }
}
