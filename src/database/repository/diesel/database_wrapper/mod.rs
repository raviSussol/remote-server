pub mod macros;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection, SqliteConnection,
};
use serde::Deserialize;

use crate::{database::repository::RepositoryError, util::settings::Settings};

#[derive(Debug, Clone, Deserialize)]
pub enum ConnectionType {
    Pg,
    Sqlite,
}

pub enum DbConnectionPool {
    Pg(Pool<ConnectionManager<PgConnection>>),
    Sqlite(Pool<ConnectionManager<SqliteConnection>>),
}

pub enum DbConnection {
    Pg(PooledConnection<ConnectionManager<PgConnection>>),
    Sqlite(PooledConnection<ConnectionManager<SqliteConnection>>),
}

impl From<r2d2::Error> for RepositoryError {
    fn from(err: r2d2::Error) -> Self {
        RepositoryError::OtherConnectionError(err.to_string())
    }
}

impl DbConnectionPool {
    pub fn new(settings: &Settings) -> DbConnectionPool {
        match settings.database.database_type {
            ConnectionType::Pg => DbConnectionPool::new_pg(&settings.database.connection_string()),
            ConnectionType::Sqlite => {
                DbConnectionPool::new_sqlite(&settings.database.connection_string())
            }
        }
    }

    pub fn new_pg(connection_string: &str) -> DbConnectionPool {
        let connection_manager = ConnectionManager::<PgConnection>::new(connection_string);
        DbConnectionPool::Pg(Pool::new(connection_manager).expect("Failed to connect to database"))
    }

    pub fn new_sqlite(connection_string: &str) -> DbConnectionPool {
        let connection_manager = ConnectionManager::<SqliteConnection>::new(connection_string);
        DbConnectionPool::Sqlite(
            Pool::new(connection_manager).expect("Failed to connect to database"),
        )
    }

    pub fn get_pg_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<diesel::PgConnection>>, RepositoryError> {
        match self {
            DbConnectionPool::Pg(pool) => Ok(pool.get()?),
            _ => Err(RepositoryError::ConnectionDoesntExist(ConnectionType::Pg)),
        }
    }

    pub fn get_sqlite_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<diesel::SqliteConnection>>, RepositoryError>
    {
        match self {
            DbConnectionPool::Sqlite(pool) => Ok(pool.get()?),
            _ => Err(RepositoryError::ConnectionDoesntExist(
                ConnectionType::Sqlite,
            )),
        }
    }

    pub fn get_connection(&self) -> Result<DbConnection, RepositoryError> {
        match self {
            DbConnectionPool::Pg(pool) => Ok(DbConnection::Pg(pool.get()?)),
            DbConnectionPool::Sqlite(pool) => Ok(DbConnection::Sqlite(pool.get()?)),
        }
    }

    pub fn clone(&self) -> DbConnectionPool {
        match self {
            DbConnectionPool::Pg(pool) => DbConnectionPool::Pg(pool.clone()),
            DbConnectionPool::Sqlite(pool) => DbConnectionPool::Sqlite(pool.clone()),
        }
    }
}
