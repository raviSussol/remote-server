use super::StorageConnection;

use crate::{repository_error::RepositoryError, schema::MasterListRow};

use diesel::prelude::*;

pub struct MasterListRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> MasterListRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        MasterListRepository { connection }
    }

    #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
    pub fn upsert_one(&self, row: &MasterListRow) -> Result<(), RepositoryError> {
        use crate::schema::diesel_schema::master_list::dsl::*;

        diesel::insert_into(master_list)
            .values(row)
            .on_conflict(id)
            .do_update()
            .set(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    pub fn upsert_one(&self, row: &MasterListRow) -> Result<(), RepositoryError> {
        use crate::schema::diesel_schema::master_list::dsl::*;
        diesel::replace_into(master_list)
            .values(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, item_id: &str) -> Result<MasterListRow, RepositoryError> {
        use crate::schema::diesel_schema::master_list::dsl::*;
        let result = master_list
            .filter(id.eq(item_id))
            .first(&self.connection.connection)?;
        Ok(result)
    }
}
