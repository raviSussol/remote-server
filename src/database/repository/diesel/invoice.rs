use super::{DBBackendConnection, DBConnection};

use crate::{
    business::FullInvoice,
    database::{
        repository::{repository::get_connection, RepositoryError},
        schema::InvoiceRow,
    },
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
        use crate::database::schema::diesel_schema::invoice::dsl::*;
        let connection = get_connection(&self.pool)?;
        diesel::insert_into(invoice)
            .values(invoice_row)
            .execute(&connection)?;
        Ok(())
    }

    pub fn one(connection: &DBConnection, invoice_id: &str) -> Result<InvoiceRow, RepositoryError> {
        use crate::database::schema::diesel_schema::invoice::dsl::*;

        invoice
            .filter(id.eq(invoice_id))
            .first(connection)
            .map_err(RepositoryError::from)
    }

    pub async fn find_one_by_id(&self, invoice_id: &str) -> Result<InvoiceRow, RepositoryError> {
        let connection = get_connection(&self.pool)?;
        InvoiceRepository::one(&connection, invoice_id)
    }

    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<InvoiceRow>, RepositoryError> {
        use crate::database::schema::diesel_schema::invoice::dsl::*;
        let connection = get_connection(&self.pool)?;
        let result = invoice.filter(id.eq_any(ids)).load(&connection)?;
        Ok(result)
    }
}

pub struct FullInvoiceRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl From<FullInvoice> for InvoiceRow {
    fn from(
        FullInvoice {
            id,
            name_id,
            store_id,
            invoice_number,
            r#type,
            status,
            comment,
            their_reference,
            entry_datetime,
            confirm_datetime,
            finalised_datetime,
        }: FullInvoice,
    ) -> Self {
        InvoiceRow {
            id,
            name_id,
            store_id,
            invoice_number,
            r#type,
            status,
            comment,
            their_reference,
            entry_datetime,
            confirm_datetime,
            finalised_datetime,
        }
    }
}

impl FullInvoiceRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> FullInvoiceRepository {
        FullInvoiceRepository { pool }
    }

    pub async fn insert(&self, invoice: FullInvoice) -> Result<(), RepositoryError> {
        use crate::database::schema::diesel_schema::invoice::dsl as invoice_dsl;
        let connection = get_connection(&self.pool)?;

        // Also insert the following in one transaction
        // stock lines
        // lines
        diesel::insert_into(invoice_dsl::invoice)
            .values(InvoiceRow::from(invoice))
            .execute(&connection)?;

        Ok(())
    }

    pub async fn update(&self, invoice: FullInvoice) -> Result<(), RepositoryError> {
        use crate::database::schema::diesel_schema::invoice::dsl as invoice_dsl;
        let connection = get_connection(&self.pool)?;

        // Also insert the following in one transaction
        // stock lines
        // lines
        diesel::update(invoice_dsl::invoice)
            .set(InvoiceRow::from(invoice))
            .execute(&connection)?;

        Ok(())
    }
}
