pub mod macros;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use serde::Deserialize;

use crate::{database::repository::RepositoryError, util::settings::Settings};

// Use sqlite instead of postgres and postgres instead of sqlite
#[cfg(feature = "postgres")]
pub type PgConnection = diesel::PgConnection;
#[cfg(not(feature = "postgres"))]
pub type PgConnection = std::marker::PhantomData<bool>;

#[cfg(feature = "sqlite")]
pub type SqliteConnection = diesel::SqliteConnection;
#[cfg(not(feature = "sqlite"))]
pub type SqliteConnection = std::marker::PhantomData<bool>;

#[cfg(feature = "postgres")]
type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;
#[cfg(not(feature = "postgres"))]
type PgPooledConnection = std::marker::PhantomData<bool>;

#[cfg(feature = "sqlite")]
type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;
#[cfg(not(feature = "sqlite"))]
type SqlitePooledConnection = std::marker::PhantomData<bool>;

#[cfg(feature = "postgres")]
type PgPool = Pool<ConnectionManager<PgConnection>>;
#[cfg(not(feature = "postgres"))]
type PgPool = std::marker::PhantomData<bool>;

#[cfg(feature = "sqlite")]
type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
#[cfg(not(feature = "sqlite"))]
type SqlitePool = std::marker::PhantomData<bool>;

#[derive(Debug, Clone, Deserialize)]
pub enum ConnectionType {
    Pg,
    Sqlite,
}

pub enum DbConnectionPool {
    Pg(PgPool),
    Sqlite(SqlitePool),
}

pub enum DbConnection {
    Pg(PgPooledConnection),
    Sqlite(SqlitePooledConnection),
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

    #[cfg(feature = "postgres")]
    pub fn new_pg(connection_string: &str) -> DbConnectionPool {
        let connection_manager = ConnectionManager::<PgConnection>::new(connection_string);
        DbConnectionPool::Pg(Pool::new(connection_manager).expect("Failed to connect to database"))
    }

    #[cfg(not(feature = "postgres"))]
    pub fn new_pg(connection_string: &str) -> DbConnectionPool {
        panic!("postgres flag is not enabled")
    }

    #[cfg(feature = "sqlite")]
    pub fn new_sqlite(connection_string: &str) -> DbConnectionPool {
        let connection_manager = ConnectionManager::<SqliteConnection>::new(connection_string);
        DbConnectionPool::Sqlite(
            Pool::new(connection_manager).expect("Failed to connect to database"),
        )
    }

    #[cfg(not(feature = "sqlite"))]
    pub fn new_sqlite(connection_string: &str) -> DbConnectionPool {
        panic!("sqlite flag is not enabled")
    }

    #[cfg(feature = "postgres")]
    pub fn get_pg_connection(&self) -> Result<PgPooledConnection, RepositoryError> {
        match self {
            DbConnectionPool::Pg(pool) => Ok(pool.get()?),
            _ => Err(RepositoryError::ConnectionDoesntExist(ConnectionType::Pg)),
        }
    }

    #[cfg(not(feature = "postgres"))]
    pub fn get_pg_connection(&self) -> Result<PgPooledConnection, RepositoryError> {
        panic!("postgres flag is not enabled")
    }

    #[cfg(feature = "sqlite")]
    pub fn get_sqlite_connection(&self) -> Result<SqlitePooledConnection, RepositoryError> {
        match self {
            DbConnectionPool::Sqlite(pool) => Ok(pool.get()?),
            _ => Err(RepositoryError::ConnectionDoesntExist(
                ConnectionType::Sqlite,
            )),
        }
    }

    #[cfg(not(feature = "sqlite"))]
    pub fn get_sqlite_connection(&self) -> Result<SqlitePooledConnection, RepositoryError> {
        panic!("sqlite flag is not enabled")
    }

    pub fn get_connection(&self) -> Result<DbConnection, RepositoryError> {
        match self {
            DbConnectionPool::Pg(_) => Ok(DbConnection::Pg(self.get_pg_connection()?)),
            DbConnectionPool::Sqlite(_) => Ok(DbConnection::Sqlite(self.get_sqlite_connection()?)),
        }
    }

    pub fn clone(&self) -> DbConnectionPool {
        match self {
            DbConnectionPool::Pg(pool) => DbConnectionPool::Pg(pool.clone()),
            DbConnectionPool::Sqlite(pool) => DbConnectionPool::Sqlite(pool.clone()),
        }
    }
}
