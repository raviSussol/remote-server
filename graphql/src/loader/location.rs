use domain::location::{Location, LocationFilter};
use repository::storage_connection_example::ConnectionPool;
use repository::{LocationRepository, RepositoryError};

use async_graphql::dataloader::*;
use async_graphql::*;
use std::collections::HashMap;

pub struct LocationByIdLoader {
    pub connection_pool: ConnectionPool,
}

#[async_trait::async_trait]
impl Loader<String> for LocationByIdLoader {
    type Value = Location;
    type Error = RepositoryError;

    async fn load(&self, ids: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let repo = LocationRepository::new(self.connection_pool.connection()?);

        let result = repo.query_filter_only(LocationFilter::new().match_ids(ids.to_owned()))?;

        Ok(result
            .into_iter()
            .map(|stock_line| (stock_line.id.clone(), stock_line))
            .collect())
    }
}
