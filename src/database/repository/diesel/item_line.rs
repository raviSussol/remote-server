use super::DBBackendConnection;

use crate::database::{
    repository::{repository::get_connection, RepositoryError},
    schema::ItemLineRow,
};

use diesel::backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax};
use diesel::connection::{AnsiTransactionManager, TransactionManager};
use diesel::deserialize::FromSql;
use diesel::query_builder::bind_collector::RawBytesBindCollector;
use diesel::query_builder::{BindCollector, QueryBuilder};
use diesel::sql_types::TypeMetadata;
use diesel::Connection;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

use byteorder::NativeEndian;

trait MyConnection: Connection
where
    Self::Backend: Backend,
{
}

impl<T> MyConnection for T
where
    T: Connection,
    T::Backend: Backend,
{
}

#[derive(Clone)]
pub struct ItemLineRepository<T: 'static + MyConnection> {
    pool: Pool<ConnectionManager<T>>,
}

impl<T: MyConnection> ItemLineRepository<T> {
    pub fn new(pool: Pool<ConnectionManager<T>>) -> Self {
        ItemLineRepository { pool }
    }

    pub async fn insert_one(&self, item_line_row: &ItemLineRow) -> Result<(), RepositoryError> {
        use crate::database::schema::diesel_schema::item_line::dsl::*;
        let connection = self.pool.get().unwrap();
        diesel::insert_into(item_line)
            .values(item_line_row)
            .execute(&connection)?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, item_line_id: &str) -> Result<ItemLineRow, RepositoryError> {
        use crate::database::schema::diesel_schema::item_line::dsl::*;
        let connection = self.pool.get().unwrap();
        let result = item_line.filter(id.eq(item_line_id)).first(&connection)?;
        Ok(result)
    }
}
