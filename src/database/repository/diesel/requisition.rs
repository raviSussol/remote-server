use crate::database::{
    repository::{
        macros::{execute, first, load},
        DbConnectionPool, RepositoryError,
    },
    schema::{diesel_schema::requisition::dsl::*, RequisitionRow},
};
use diesel::prelude::*;
pub struct RequisitionRepository {
    pool: DbConnectionPool,
}

impl RequisitionRepository {
    pub fn new(pool: DbConnectionPool) -> RequisitionRepository {
        RequisitionRepository { pool }
    }

    pub async fn insert_one(
        &self,
        requisition_row: &RequisitionRow,
    ) -> Result<(), RepositoryError> {
        execute!(
            self.pool,
            diesel::insert_into(requisition).values(requisition_row)
        )?;
        Ok(())
    }

    pub async fn find_one_by_id(
        &self,
        requisition_id: &str,
    ) -> Result<RequisitionRow, RepositoryError> {
        first!(self.pool, requisition.filter(id.eq(requisition_id)))
    }

    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<RequisitionRow>, RepositoryError> {
        load!(self.pool, requisition.filter(id.eq_any(ids)))
    }
}
