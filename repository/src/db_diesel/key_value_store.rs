use diesel::prelude::*;

use super::{key_value_store::key_value_store::dsl as kv_store_dsl, StorageConnection};
use crate::repository_error::RepositoryError;

use diesel_derive_enum::DbEnum;

table! {
    key_value_store (id) {
        id -> crate::db_diesel::key_value_store::KeyValueTypeMapping,
        value_string -> Nullable<Text>,
        value_int-> Nullable<Integer>,
        value_bigint-> Nullable<BigInt>,
        value_float-> Nullable<Double>,
        value_bool-> Nullable<Bool>,
    }
}

#[derive(DbEnum, Debug, Clone, PartialEq, Eq)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum KeyValueType {
    CentralSyncPullCursor,
    /// Indicates if the sync queue on the remote server has been initialised
    RemoteSyncInitilisationStarted,
    /// Indicates if the remote data has been pulled and integrated from the central server
    /// Possible value: "true"
    RemoteSyncInitilisationFinished,
    RemoteSyncPushCursor,

    SettingsSyncUrl,
    SettingsSyncUsername,
    SettingsSyncPasswordSha256,
    SettingsSyncIntervalSec,
    SettingsSyncCentralServerSiteId,
    SettingsSyncSideId,
    SettingsSyncSideHardwareId,
}

#[derive(Clone, Queryable, Insertable, AsChangeset, Debug, PartialEq)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "key_value_store"]
pub struct KeyValueStoreRow {
    pub id: KeyValueType,
    pub value_string: Option<String>,
    pub value_int: Option<i32>,
    pub value_bigint: Option<i64>,
    pub value_float: Option<f64>,
    pub value_bool: Option<bool>,
}

pub struct KeyValueStoreRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> KeyValueStoreRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        KeyValueStoreRepository { connection }
    }

    #[cfg(feature = "postgres")]
    pub fn upsert_one(&self, value: &KeyValueStoreRow) -> Result<(), RepositoryError> {
        diesel::insert_into(kv_store_dsl::key_value_store)
            .values(value)
            .on_conflict(kv_store_dsl::id)
            .do_update()
            .set(value)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    #[cfg(not(feature = "postgres"))]
    pub fn upsert_one(&self, value: &KeyValueStoreRow) -> Result<(), RepositoryError> {
        diesel::replace_into(kv_store_dsl::key_value_store)
            .values(value)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    fn get_row(&self, key: KeyValueType) -> Result<Option<KeyValueStoreRow>, RepositoryError> {
        let result = kv_store_dsl::key_value_store
            .filter(kv_store_dsl::id.eq(key))
            .first(&self.connection.connection)
            .optional()?;
        Ok(result)
    }

    pub fn set_string(
        &self,
        key: KeyValueType,
        value: Option<String>,
    ) -> Result<(), RepositoryError> {
        self.upsert_one(&KeyValueStoreRow {
            id: key,
            value_string: value,
            value_int: None,
            value_bigint: None,
            value_float: None,
            value_bool: None,
        })
    }

    pub fn get_string(&self, key: KeyValueType) -> Result<Option<String>, RepositoryError> {
        let row = self.get_row(key)?;
        Ok(row.and_then(|row| row.value_string))
    }

    pub fn set_i32(&self, key: KeyValueType, value: Option<i32>) -> Result<(), RepositoryError> {
        self.upsert_one(&KeyValueStoreRow {
            id: key,
            value_string: None,
            value_int: value,
            value_bigint: None,
            value_float: None,
            value_bool: None,
        })
    }

    pub fn get_i32(&self, key: KeyValueType) -> Result<Option<i32>, RepositoryError> {
        let row = self.get_row(key)?;
        Ok(row.and_then(|row| row.value_int))
    }

    pub fn set_i64(&self, key: KeyValueType, value: Option<i64>) -> Result<(), RepositoryError> {
        self.upsert_one(&KeyValueStoreRow {
            id: key,
            value_string: None,
            value_int: None,
            value_bigint: value,
            value_float: None,
            value_bool: None,
        })
    }

    pub fn get_i64(&self, key: KeyValueType) -> Result<Option<i64>, RepositoryError> {
        let row = self.get_row(key)?;
        Ok(row.and_then(|row| row.value_bigint))
    }

    pub fn set_f64(&self, key: KeyValueType, value: Option<f64>) -> Result<(), RepositoryError> {
        self.upsert_one(&KeyValueStoreRow {
            id: key,
            value_string: None,
            value_int: None,
            value_bigint: None,
            value_float: value,
            value_bool: None,
        })
    }

    pub fn get_f64(&self, key: KeyValueType) -> Result<Option<f64>, RepositoryError> {
        let row = self.get_row(key)?;
        Ok(row.and_then(|row| row.value_float))
    }

    pub fn set_bool(&self, key: KeyValueType, value: Option<bool>) -> Result<(), RepositoryError> {
        self.upsert_one(&KeyValueStoreRow {
            id: key,
            value_string: None,
            value_int: None,
            value_bigint: None,
            value_float: None,
            value_bool: value,
        })
    }

    pub fn get_bool(&self, key: KeyValueType) -> Result<Option<bool>, RepositoryError> {
        let row = self.get_row(key)?;
        Ok(row.and_then(|row| row.value_bool))
    }
}
