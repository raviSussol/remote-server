use crate::database::{
    repository::{
        macros::{execute_pool, first_pool, get_results_pool, load_pool},
        DbConnectionPool, RepositoryError,
    },
    schema::{diesel_schema::transact::dsl::*, TransactRow, TransactRowType},
};

use diesel::prelude::*;

pub struct TransactRepository {
    pool: DbConnectionPool,
}

impl TransactRepository {
    pub fn new(pool: DbConnectionPool) -> TransactRepository {
        TransactRepository { pool }
    }

    pub async fn insert_one(&self, transact_row: &TransactRow) -> Result<(), RepositoryError> {
        execute_pool!(
            self.pool,
            diesel::insert_into(transact).values(transact_row)
        )?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, transact_id: &str) -> Result<TransactRow, RepositoryError> {
        first_pool!(self.pool, transact.filter(id.eq(transact_id)))
    }

    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<TransactRow>, RepositoryError> {
        load_pool!(self.pool, transact.filter(id.eq_any(ids)))
    }
}

pub struct CustomerInvoiceRepository {
    pool: DbConnectionPool,
}

impl CustomerInvoiceRepository {
    pub fn new(pool: DbConnectionPool) -> CustomerInvoiceRepository {
        CustomerInvoiceRepository { pool }
    }

    pub async fn find_many_by_name_id(
        &self,
        name: &str,
    ) -> Result<Vec<TransactRow>, RepositoryError> {
        get_results_pool!(
            self.pool,
            transact.filter(
                type_of
                    .eq(TransactRowType::CustomerInvoice)
                    .and(name_id.eq(name))
            )
        )
    }

    pub async fn find_many_by_store_id(
        &self,
        store: &str,
    ) -> Result<Vec<TransactRow>, RepositoryError> {
        get_results_pool!(
            self.pool,
            transact.filter(
                type_of
                    .eq(TransactRowType::CustomerInvoice)
                    .and(store_id.eq(store)),
            )
        )
    }
}
