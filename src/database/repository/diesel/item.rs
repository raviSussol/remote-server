use super::DBBackendConnection;

use crate::database::{
    repository::{repository::get_connection, RepositoryError},
    schema::{ItemRow, ItemRowType},
};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use r2d2::PooledConnection;

pub fn example(pool: &Pool<ConnectionManager<DBBackendConnection>>) -> Result<(), RepositoryError> {
    let connection = &get_connection(pool)?;
    connection
        .transaction::<(), RepositoryError, _>(|| {
            let row = ItemRow {
                id: "abc".to_owned(),
                item_name: "amox".to_owned(),
                type_of: ItemRowType::General,
            };

            ItemRepository::insert_one_sync(&row, connection)?;

            let row = ItemRow {
                id: "abc".to_owned(),
                item_name: "amox".to_owned(),
                type_of: ItemRowType::General,
            };

            ItemRepository::insert_one_sync(&row, connection)?;

            // etc ofcourse above would be similar to import_sync_records, but no need for async etc..

            Ok(())
        })
        .unwrap();

    Ok(())
}

#[derive(Clone)]
pub struct ItemRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl ItemRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> ItemRepository {
        ItemRepository { pool: pool }
    }

    pub async fn insert_one(&self, item_row: &ItemRow) -> Result<(), RepositoryError> {
        ItemRepository::insert_one_sync(item_row, &get_connection(&self.pool)?)
    }

    pub fn insert_one_sync(
        item_row: &ItemRow,
        pool: &PooledConnection<ConnectionManager<DBBackendConnection>>,
    ) -> Result<(), RepositoryError> {
        use crate::database::schema::diesel_schema::item::dsl::*;

        diesel::insert_into(item).values(item_row).execute(pool)?;

        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<ItemRow>, RepositoryError> {
        use crate::database::schema::diesel_schema::item::dsl::*;
        let connection = get_connection(&self.pool)?;
        let result = item.load(&connection);
        Ok(result?)
    }

    pub async fn find_one_by_id(&self, item_id: &str) -> Result<ItemRow, RepositoryError> {
        use crate::database::schema::diesel_schema::item::dsl::*;
        let connection = get_connection(&self.pool)?;
        let result = item.filter(id.eq(item_id)).first(&connection)?;
        Ok(result)
    }
}
