use crate::database::repository::{ItemRepository, RepositoryError, StorageConnectionManager};
use crate::database::schema::ItemRow;

use async_graphql::dataloader::*;
use async_graphql::*;
use std::collections::HashMap;

pub struct ItemLoader {
    pub connection_manager: StorageConnectionManager,
}

#[async_trait::async_trait]
impl Loader<String> for ItemLoader {
    type Value = ItemRow;
    type Error = RepositoryError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let con = self.connection_manager.new_storage_context()?;
        let repo = ItemRepository::new(&con);
        let result = repo
            .find_many_by_id(keys)
            .await
            .unwrap()
            .iter()
            .map(|item: &ItemRow| {
                let item_id = item.id.clone();
                let item = item.clone();
                (item_id, item)
            })
            .collect();
        Ok(result)
    }
}
