use crate::database::{
    repository::{
        macros::{execute, first, get_results, load},
        RepositoryError,
    },
    schema::{diesel_schema::transact_line::dsl::*, TransactLineRow},
};
use diesel::prelude::*;

use super::DbConnectionPool;

pub struct TransactLineRepository {
    pool: DbConnectionPool,
}

impl TransactLineRepository {
    pub fn new(pool: DbConnectionPool) -> TransactLineRepository {
        TransactLineRepository { pool }
    }

    pub async fn insert_one(
        &self,
        transact_line_row: &TransactLineRow,
    ) -> Result<(), RepositoryError> {
        execute!(
            self.pool,
            diesel::insert_into(transact_line).values(transact_line_row)
        )?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, row_id: &str) -> Result<TransactLineRow, RepositoryError> {
        first!(self.pool, transact_line.filter(id.eq(row_id)))
    }

    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<TransactLineRow>, RepositoryError> {
        load!(self.pool, transact_line.filter(id.eq_any(ids)))
    }

    pub async fn find_many_by_transact_id(
        &self,
        trans_id: &str,
    ) -> Result<Vec<TransactLineRow>, RepositoryError> {
        get_results!(self.pool, transact_line.filter(transact_id.eq(trans_id)))
    }
}
