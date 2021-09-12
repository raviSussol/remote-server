use crate::database::{
    repository::{
        macros::transaction, DbConnectionPool, ItemRepository, NameRepository, RepositoryError,
    },
    schema::{ItemRow, NameRow},
};

use diesel::prelude::*;

pub enum IntegrationUpsertRecord {
    Name(NameRow),
    Item(ItemRow),
}

pub struct IntegrationRecord {
    pub upserts: Vec<IntegrationUpsertRecord>,
}

pub struct SyncRepository {
    pool: DbConnectionPool,
}

impl SyncRepository {
    pub fn new(pool: DbConnectionPool) -> SyncRepository {
        SyncRepository { pool }
    }

    pub async fn integrate_records(
        &self,
        integration_records: &IntegrationRecord,
    ) -> Result<(), RepositoryError> {
        let connection = self.pool.get_connection()?;
        transaction!(&connection, || {
            for record in &integration_records.upserts {
                match &record {
                    IntegrationUpsertRecord::Name(record) => {
                        NameRepository::upsert_one_tx(&connection, record)?
                    }
                    IntegrationUpsertRecord::Item(record) => {
                        ItemRepository::upsert_one_tx(&connection, record)?
                    }
                }
            }
            Ok(())
        })
    }
}
