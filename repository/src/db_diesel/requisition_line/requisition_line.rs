use crate::{
    diesel_macros::apply_equal_filter,
    repository_error::RepositoryError,
    schema::{
        diesel_schema::{requisition_line, requisition_line::dsl as requisition_line_dsl},
        ItemRow, RequisitionLineRow,
    },
    DBType, StorageConnection,
};

use diesel::prelude::*;
use domain::Pagination;

use super::RequisitionLineFilter;

pub type RequisitionLineJoin = (RequisitionLineRow, ItemRow);
pub struct RequisitionLine {
    pub requisition_line_row: RequisitionLineRow,
}

pub struct RequisitionLineRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> RequisitionLineRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        RequisitionLineRepository { connection }
    }

    pub fn count(&self, filter: Option<RequisitionLineFilter>) -> Result<i64, RepositoryError> {
        let query = create_filtered_query(filter)?;
        Ok(query.count().get_result(&self.connection.connection)?)
    }

    pub fn query_by_filter(
        &self,
        filter: RequisitionLineFilter,
    ) -> Result<Vec<RequisitionLine>, RepositoryError> {
        self.query(Pagination::new(), Some(filter))
    }

    pub fn query(
        &self,
        pagination: Pagination,
        filter: Option<RequisitionLineFilter>,
    ) -> Result<Vec<RequisitionLine>, RepositoryError> {
        let mut query = create_filtered_query(filter)?;

        query = query.order(requisition_line_dsl::id.asc());

        let result = query
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64)
            .load::<RequisitionLineRow>(&self.connection.connection)?;

        Ok(result
            .into_iter()
            .map(|requisition_line_row| RequisitionLine {
                requisition_line_row,
            })
            .collect())
    }
}

type BoxedRequisitionLineQuery = requisition_line::BoxedQuery<'static, DBType>;

fn create_filtered_query(
    filter: Option<RequisitionLineFilter>,
) -> Result<BoxedRequisitionLineQuery, RepositoryError> {
    let mut query = requisition_line_dsl::requisition_line.into_boxed();

    if let Some(f) = filter {
        apply_equal_filter!(query, f.id, requisition_line_dsl::id);
        apply_equal_filter!(
            query,
            f.requisition_id,
            requisition_line_dsl::requisition_id
        );
        apply_equal_filter!(
            query,
            f.requested_quantity,
            requisition_line_dsl::requested_quantity
        );
    }

    Ok(query)
}