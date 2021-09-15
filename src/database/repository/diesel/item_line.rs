use crate::database::{
    repository::{
        macros::{execute, first, load},
        DbConnectionPool, RepositoryError,
    },
    schema::{diesel_schema::item_line::dsl::*, ItemLineRow},
};
use diesel::prelude::*;
pub struct ItemLineRepository {
    pool: DbConnectionPool,
}

impl ItemLineRepository {
    pub fn new(pool: DbConnectionPool) -> Self {
        ItemLineRepository { pool }
    }

    pub async fn insert_one(&self, item_line_row: &ItemLineRow) -> Result<(), RepositoryError> {
        execute!(
            self.pool,
            diesel::insert_into(item_line).values(item_line_row)
        )?;

        Ok(())
    }

    pub async fn find_one_by_id(&self, item_line_id: &str) -> Result<ItemLineRow, RepositoryError> {
        first!(self.pool, item_line.filter(id.eq(item_line_id)))
    }

    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<ItemLineRow>, RepositoryError> {
        load!(self.pool, item_line.filter(id.eq_any(ids)))
    }
}
