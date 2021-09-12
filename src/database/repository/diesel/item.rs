use crate::database::{
    repository::{
        macros::{execute_connection, first_pool, load_pool},
        DbConnection, DbConnectionPool, RepositoryError,
    },
    schema::{diesel_schema::item::dsl::*, ItemRow},
};

use diesel::prelude::*;
pub struct ItemRepository {
    pool: DbConnectionPool,
}

impl ItemRepository {
    pub fn new(pool: DbConnectionPool) -> ItemRepository {
        ItemRepository { pool }
    }

    pub fn upsert_one_tx(
        connection: &DbConnection,
        item_row: &ItemRow,
    ) -> Result<(), RepositoryError> {
        execute_connection!(
            connection,
            // Postgres
            diesel::insert_into(item)
                .values(item_row)
                .on_conflict(id)
                .do_update()
                .set(item_row),
            // Sqlite
            diesel::replace_into(item).values(item_row)
        )?;

        Ok(())
    }

    pub fn insert_one_tx(
        connection: &DbConnection,
        item_row: &ItemRow,
    ) -> Result<(), RepositoryError> {
        execute_connection!(connection, diesel::insert_into(item).values(item_row))?;
        Ok(())
    }

    pub async fn insert_one(&self, item_row: &ItemRow) -> Result<(), RepositoryError> {
        ItemRepository::insert_one_tx(&self.pool.get_connection()?, item_row)
    }

    pub async fn find_all(&self) -> Result<Vec<ItemRow>, RepositoryError> {
        load_pool!(self.pool, item)
    }

    pub async fn find_one_by_id(&self, item_id: &str) -> Result<ItemRow, RepositoryError> {
        first_pool!(self.pool, item.filter(id.eq(item_id)))
    }

    pub async fn find_many_by_id(&self, ids: &[String]) -> Result<Vec<ItemRow>, RepositoryError> {
        load_pool!(self.pool, item.filter(id.eq_any(ids)))
    }
}
