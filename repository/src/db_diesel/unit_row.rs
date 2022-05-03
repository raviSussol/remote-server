use super::StorageConnection;
use crate::{db_diesel::unit_row::unit::dsl::*, repository_error::RepositoryError};
use diesel::prelude::*;

table! {
    unit (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        index -> Integer,
    }
}

#[derive(Clone, Insertable, Queryable, Debug, PartialEq, Eq, AsChangeset)]
#[table_name = "unit"]
pub struct UnitRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub index: i32,
}

pub struct UnitRowRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> UnitRowRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        UnitRowRepository { connection }
    }

    #[cfg(feature = "postgres")]
    pub fn upsert_one(&self, row: &UnitRow) -> Result<(), RepositoryError> {
        diesel::insert_into(unit)
            .values(row)
            .on_conflict(id)
            .do_update()
            .set(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    #[cfg(not(feature = "postgres"))]
    pub fn upsert_one(&self, row: &UnitRow) -> Result<(), RepositoryError> {
        diesel::replace_into(unit)
            .values(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, unit_id: &str) -> Result<UnitRow, RepositoryError> {
        let result = unit
            .filter(id.eq(unit_id))
            .first(&self.connection.connection)?;
        Ok(result)
    }
}
