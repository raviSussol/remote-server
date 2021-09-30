use super::DBBackendConnection;

use crate::{
    business::{FullInvoice, FullInvoiceLine, FullInvoiceMutation},
    database::{
        repository::{repository::get_connection, RepositoryError},
        schema::{InvoiceLineRow, InvoiceRow, StockLineRow},
    },
};

use crate::database::schema::diesel_schema::{
    invoice::dsl as invoice_dsl, invoice_line::dsl as invoice_line_dsl,
    stock_line::dsl as stock_line_dsl,
};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

pub struct InvoiceRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl InvoiceRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> InvoiceRepository {
        InvoiceRepository { pool }
    }

    pub async fn insert_one(&self, invoice_row: &InvoiceRow) -> Result<(), RepositoryError> {
        use self::invoice_dsl::*;

        let connection = get_connection(&self.pool)?;
        diesel::insert_into(invoice)
            .values(invoice_row)
            .execute(&connection)?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, invoice_id: &str) -> Result<InvoiceRow, RepositoryError> {
        use self::invoice_dsl::*;

        let connection = get_connection(&self.pool)?;
        invoice
            .filter(id.eq(invoice_id))
            .first(&connection)
            .map_err(RepositoryError::from)
    }

    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<InvoiceRow>, RepositoryError> {
        use self::invoice_dsl::*;

        let connection = get_connection(&self.pool)?;
        let result = invoice.filter(id.eq_any(ids)).load(&connection)?;
        Ok(result)
    }
}

pub struct FullInvoiceRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

type InvoiceLinesWithStockLine = (InvoiceLineRow, Option<StockLineRow>);

impl From<InvoiceLinesWithStockLine> for FullInvoiceLine {
    fn from((invoice_line_row, stock_line_row): InvoiceLinesWithStockLine) -> Self {
        FullInvoiceLine {
            line: invoice_line_row,
            batch: stock_line_row,
        }
    }
}

impl FullInvoiceRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> FullInvoiceRepository {
        FullInvoiceRepository { pool }
    }

    pub async fn one(&self, invoice_id: &str) -> Result<FullInvoice, RepositoryError> {
        let connection = get_connection(&self.pool)?;
        let invoice: InvoiceRow = invoice_dsl::invoice
            .filter(invoice_dsl::id.eq(invoice_id))
            .first(&connection)?;

        let invoice_lines_with_stock_line: Vec<InvoiceLinesWithStockLine> =
            invoice_line_dsl::invoice_line
                .left_join(stock_line_dsl::stock_line)
                .filter(invoice_line_dsl::invoice_id.eq(invoice_id))
                .load(&connection)?;

        Ok(FullInvoice {
            invoice,
            lines: invoice_lines_with_stock_line
                .into_iter()
                .map(FullInvoiceLine::from)
                .collect(),
        })
    }

    pub async fn mutate(
        &self,
        FullInvoiceMutation {
            invoice,
            lines,
            batches,
        }: FullInvoiceMutation,
    ) -> Result<(), RepositoryError> {
        todo!()
        // let connection = get_connection(&self.pool)?;

        // connection.transaction::<(), RepositoryError, _>(|| {
        //     // Inserts
        //     if let Some(inserts) = invoice.inserts {
        //         diesel::insert_into(invoice_dsl::invoice)
        //             .values(inserts)
        //             .execute(&*connection)?;
        //     }

        //     if let Some(inserts) = batches.inserts {
        //         diesel::insert_into(stock_line_dsl::stock_line)
        //             .values(inserts)
        //             .execute(&*connection)?;
        //     }

        //     if let Some(inserts) = lines.inserts {
        //         diesel::insert_into(invoice_line_dsl::invoice_line)
        //             .values(inserts)
        //             .execute(&*connection)?;
        //     }

        //     // Updates
        //     if let Some(updates) = batches.updates {
        //         for update in updates.into_iter() {
        //             diesel::update(stock_line_dsl::stock_line)
        //                 .set(update)
        //                 .execute(&connection)?;
        //         }
        //     }

        //     if let Some(updates) = lines.updates {
        //         for update in updates.into_iter() {
        //             diesel::update(invoice_line_dsl::invoice_line)
        //                 .set(update)
        //                 .execute(&connection)?;
        //         }
        //     }

        //     if let Some(updates) = invoice.updates {
        //         for update in updates.into_iter() {
        //             diesel::update(invoice_dsl::invoice)
        //                 .set(update)
        //                 .execute(&connection)?;
        //         }
        //     }

        //     // Deletes

        //     if let Some(deletes) = invoice.deletes {
        //         diesel::delete(invoice_dsl::invoice.filter(invoice_dsl::id.eq_any(deletes)))
        //             .execute(&connection)?;
        //     }

        //     if let Some(deletes) = lines.deletes {
        //         diesel::delete(
        //             invoice_line_dsl::invoice_line.filter(invoice_line_dsl::id.eq_any(deletes)),
        //         )
        //         .execute(&connection)?;
        //     }

        //     if let Some(deletes) = batches.deletes {
        //         diesel::delete(
        //             stock_line_dsl::stock_line.filter(stock_line_dsl::id.eq_any(deletes)),
        //         )
        //         .execute(&connection)?;
        //     }
        //     Ok(())
        //  }//)
    }
}
