use crate::{
    database::repository::{DBConnectionPool, ItemQueryRepository},
    domain::{
        item::{Item, ItemFilter, ItemSort},
        PaginationOption,
    },
};

use super::{get_default_pagination, i64_to_u32, ListError, ListResult};

pub const MAX_LIMIT: u32 = 1000;
pub const MIN_LIMIT: u32 = 1;

pub fn get_items(
    connection_pool: &DBConnectionPool,
    pagination: Option<PaginationOption>,
    filter: Option<ItemFilter>,
    sort: Option<ItemSort>,
) -> Result<ListResult<Item>, ListError> {
    let pagination = get_default_pagination(pagination, MAX_LIMIT, MIN_LIMIT)?;
    let repository = ItemQueryRepository::new(connection_pool.clone());

    Ok(ListResult {
        rows: repository.query(pagination, &filter, sort)?,
        count: i64_to_u32(repository.count(&filter)?),
    })
}
