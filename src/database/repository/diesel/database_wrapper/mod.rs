pub mod macros;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{result::Error, Connection};
use serde::Deserialize;

use crate::{database::repository::RepositoryError, util::settings::Settings};

#[cfg(feature = "postgres")]
use diesel::PgConnection;

#[cfg(feature = "sqlite")]
use diesel::SqliteConnection;

// When we compile without postgres support or without sqlite support, ConnectionType and anything
// touching it (which largely means just DbConnectionPool and DatabaseSettings.database_type) are
// the ONLY places where the disabled database should appear in the compiled result. Everything
// else should be cfg-gated out of existence, so that we can guarantee proper access.
#[derive(Debug, Clone, Deserialize)]
pub enum ConnectionType {
    Pg,
    Sqlite,
}

#[derive(Clone)]
pub enum DbConnectionPool {
    #[cfg(feature = "postgres")]
    Pg(Pool<ConnectionManager<PgConnection>>),
    #[cfg(feature = "sqlite")]
    Sqlite(Pool<ConnectionManager<SqliteConnection>>),
}

pub enum DbConnection {
    #[cfg(feature = "postgres")]
    Pg(PooledConnection<ConnectionManager<PgConnection>>),
    #[cfg(feature = "sqlite")]
    Sqlite(PooledConnection<ConnectionManager<SqliteConnection>>),
}

impl From<r2d2::Error> for RepositoryError {
    fn from(err: r2d2::Error) -> Self {
        RepositoryError::OtherConnectionError(err.to_string())
    }
}

impl DbConnection {
    pub fn transaction<T, E, F>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: From<Error>,
    {
        match self {
            #[cfg(feature = "sqlite")]
            DbConnection::Sqlite(connection) => connection.transaction(f),
            #[cfg(feature = "postgres")]
            DbConnection::Pg(connection) => connection.transaction(f),
        }
    }
}

impl DbConnectionPool {
    pub fn new(settings: &Settings) -> DbConnectionPool {
        let connection_string = &settings.database.connection_string();
        match settings.database.database_type {
            #[cfg(feature = "postgres")]
            ConnectionType::Pg => {
                let manager = ConnectionManager::new(connection_string);
                DbConnectionPool::Pg(Pool::new(manager).expect("Failed to connect to database"))
            }
            #[cfg(not(feature = "postgres"))]
            ConnectionType::Pg => panic!("not compiled with postgres support"),
            #[cfg(feature = "sqlite")]
            ConnectionType::Sqlite => {
                let manager = ConnectionManager::new(connection_string);
                DbConnectionPool::Sqlite(Pool::new(manager).expect("Failed to connect to database"))
            }
            #[cfg(not(feature = "sqlite"))]
            ConnectionType::Sqlite => panic!("not compiled with sqlite support"),
        }
    }

    pub fn get_connection(&self) -> Result<DbConnection, RepositoryError> {
        match self {
            #[cfg(feature = "postgres")]
            DbConnectionPool::Pg(pool) => Ok(DbConnection::Pg(pool.get()?)),
            #[cfg(feature = "sqlite")]
            DbConnectionPool::Sqlite(pool) => Ok(DbConnection::Sqlite(pool.get()?)),
        }
    }
}
