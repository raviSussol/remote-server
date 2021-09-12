use crate::database::{
    repository::{
        macros::{execute_pool, first_pool, load_pool},
        DbConnectionPool, RepositoryError,
    },
    schema::{diesel_schema::requisition_line::dsl::*, RequisitionLineRow},
};
use diesel::prelude::*;

pub struct RequisitionLineRepository {
    pool: DbConnectionPool,
}

impl RequisitionLineRepository {
    pub fn new(pool: DbConnectionPool) -> RequisitionLineRepository {
        RequisitionLineRepository { pool }
    }

    pub async fn insert_one(
        &self,
        requisition_line_row: &RequisitionLineRow,
    ) -> Result<(), RepositoryError> {
        execute_pool!(
            self.pool,
            diesel::insert_into(requisition_line).values(requisition_line_row)
        )?;
        Ok(())
    }

    pub async fn find_one_by_id(
        &self,
        row_id: &str,
    ) -> Result<RequisitionLineRow, RepositoryError> {
        first_pool!(self.pool, requisition_line.filter(id.eq(row_id)))
    }

    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<RequisitionLineRow>, RepositoryError> {
        load_pool!(self.pool, requisition_line.filter(id.eq_any(ids)))
    }

    pub async fn find_many_by_requisition_id(
        &self,
        req_id: &str,
    ) -> Result<Vec<RequisitionLineRow>, RepositoryError> {
        load_pool!(
            self.pool,
            requisition_line.filter(requisition_id.eq(req_id))
        )
    }
}
