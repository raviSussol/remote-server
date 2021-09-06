use super::{DBBackendConnection, StoreRepository};

use crate::database::{
    repository::{repository::get_connection, RepositoryError},
    schema::{ItemRow, ItemRowType, StoreRow},
};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use r2d2::PooledConnection;

pub fn example(sync_repo: &SyncIntegrationRepository) -> Result<(), RepositoryError> {
    // Collect translated record

    let records = vec![
        IntegrationRecord {
            r#type: SyncType::Upsert,
            record: Record::Item(ItemRow {
                id: "abc".to_owned(),
                item_name: "amox".to_owned(),
                type_of: ItemRowType::General,
            }),
        },
        IntegrationRecord {
            r#type: SyncType::Upsert,
            record: Record::Store(StoreRow {
                id: "ABC".to_owned(),
                name_id: "CBA".to_owned(),
            }),
        },
    ];

    // then execute transction
    sync_repo.integrate_records(records)
}

enum SyncType {
    Delete,
    Upsert,
}

enum Record {
    Item(ItemRow),
    Store(StoreRow),
}

struct IntegrationRecord {
    r#type: SyncType,
    record: Record,
}

pub struct SyncIntegrationRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl SyncIntegrationRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> SyncIntegrationRepository {
        SyncIntegrationRepository { pool }
    }

    pub fn integrate_records(
        &self,
        integration_records: Vec<IntegrationRecord>,
    ) -> Result<(), RepositoryError> {
        let connection = &get_connection(&self.pool)?;
        connection.transaction::<(), RepositoryError, _>(|| {
            for record in &integration_records {
                match &record.record {
                    Record::Item(record) => ItemRepository::insert_one_sync(record, connection)?,
                    Record::Store(record) => StoreRepository::insert_one_sync(record, connection)?,
                }
            }
            Ok(())
        })
    }
}

#[derive(Clone)]
pub struct ItemRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl ItemRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> ItemRepository {
        ItemRepository { pool }
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
