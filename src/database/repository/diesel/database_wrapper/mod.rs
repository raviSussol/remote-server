pub mod macros;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use serde::Deserialize;

use crate::{database::repository::RepositoryError, util::settings::Settings};

#[cfg(feature = "postgres")]
pub use diesel::PgConnection;

#[cfg(feature = "sqlite")]
pub type SqliteConnection = diesel::SqliteConnection;

#[cfg(feature = "postgres")]
type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

#[cfg(feature = "sqlite")]
type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

#[cfg(feature = "postgres")]
type PgPool = Pool<ConnectionManager<PgConnection>>;

#[cfg(feature = "sqlite")]
type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

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
    Pg(PgPool),
    #[cfg(feature = "sqlite")]
    Sqlite(SqlitePool),
}

pub enum DbConnection {
    #[cfg(feature = "postgres")]
    Pg(PgPooledConnection),
    #[cfg(feature = "sqlite")]
    Sqlite(SqlitePooledConnection),
}

impl From<r2d2::Error> for RepositoryError {
    fn from(err: r2d2::Error) -> Self {
        RepositoryError::OtherConnectionError(err.to_string())
    }
}

impl DbConnectionPool {
    pub fn new(settings: &Settings) -> DbConnectionPool {
        let connection_string = &settings.database.connection_string();
        match settings.database.database_type {
            #[cfg(feature = "postgres")]
            ConnectionType::Pg => {
                let manager = ConnectionManager::<PgConnection>::new(connection_string);
                DbConnectionPool::Pg(Pool::new(manager).expect("Failed to connect to database"))
            }
            #[cfg(not(feature = "postgres"))]
            ConnectionType::Pg => panic!("not compiled with postgres support"),
            #[cfg(feature = "sqlite")]
            ConnectionType::Sqlite => {
                let manager = ConnectionManager::<SqliteConnection>::new(connection_string);
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
