use crate::database::{
    repository::{
        macros::{execute_pool, first_pool, load_pool},
        DbConnectionPool, RepositoryError,
    },
    schema::{diesel_schema::store::dsl::*, StoreRow},
};
use diesel::prelude::*;
pub struct StoreRepository {
    pool: DbConnectionPool,
}

impl StoreRepository {
    pub fn new(pool: DbConnectionPool) -> StoreRepository {
        StoreRepository { pool }
    }

    pub async fn insert_one(&self, store_row: &StoreRow) -> Result<(), RepositoryError> {
        execute_pool!(self.pool, diesel::insert_into(store).values(store_row))?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, store_id: &str) -> Result<StoreRow, RepositoryError> {
        first_pool!(self.pool, store.filter(id.eq(store_id)))
    }

    pub async fn find_many_by_id(&self, ids: &[String]) -> Result<Vec<StoreRow>, RepositoryError> {
        load_pool!(self.pool, store.filter(id.eq_any(ids)))
    }
}
