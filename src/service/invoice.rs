use crate::{
    database::repository::{DBConnectionPool, InvoiceQueryRepository},
    domain::{
        invoice::{Invoice, InvoiceFilter, InvoiceSort},
        PaginationOption,
    },
};

use super::{get_default_pagination, i64_to_u32, ListError, ListResult};

pub const MAX_LIMIT: u32 = 1000;
pub const MIN_LIMIT: u32 = 1;

pub fn get_invoices(
    connection_pool: &DBConnectionPool,
    pagination: Option<PaginationOption>,
    filter: Option<InvoiceFilter>,
    sort: Option<InvoiceSort>,
) -> Result<ListResult<Invoice>, ListError> {
    let pagination = get_default_pagination(pagination, MAX_LIMIT, MIN_LIMIT)?;
    let repository = InvoiceQueryRepository::new(connection_pool.clone());

    Ok(ListResult {
        rows: repository.query(pagination, &filter, sort)?,
        count: i64_to_u32(repository.count(&filter)?),
    })
}
