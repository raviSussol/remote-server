use super::{get_connection, DBBackendConnection, DBConnection};
use crate::repository_error::RepositoryError;
use diesel::{
    connection::TransactionManager,
    r2d2::{ConnectionManager as DieselConnectionManager, Pool},
    Connection as DieselConnection,
};

pub enum Connection<'a> {
    AsInstance(DBConnection),
    AsRef(&'a DBConnection),
}

impl<'a> Connection<'a> {
    pub fn duplicate(&self) -> Connection {
        use Connection::*;
        match self {
            AsInstance(connection) => AsRef(connection),
            AsRef(connection) => AsRef(connection),
        }
    }

    pub fn diesel_pooled_connection(&self) -> &DBConnection {
        match self {
            Connection::AsInstance(connection) => &connection,
            Connection::AsRef(connection) => connection,
        }
    }

    pub fn diesel_connection(&self) -> &DBBackendConnection {
        match self {
            Connection::AsInstance(connection) => &connection,
            Connection::AsRef(connection) => connection,
        }
    }
}

#[derive(Clone)]
pub struct ConnectionPool {
    diesel_connection_pool: Pool<DieselConnectionManager<DBBackendConnection>>,
}

impl<'a> ConnectionPool {
    pub fn new(diesel_connection_pool: Pool<DieselConnectionManager<DBBackendConnection>>) -> Self {
        ConnectionPool {
            diesel_connection_pool,
        }
    }

    pub fn connection_manager(&self) -> ConnectionManager<'a> {
        ConnectionManager::Pool(self.clone())
    }

    pub fn connection(&self) -> Result<Connection, RepositoryError> {
        Ok(Connection::AsInstance(get_connection(
            &self.diesel_connection_pool,
        )?))
    }
}

pub enum ConnectionManager<'a> {
    Pool(ConnectionPool),
    Connection(Connection<'a>),
}

impl<'a> ConnectionManager<'a> {
    pub fn connection(&'a self) -> Result<Connection, RepositoryError> {
        match self {
            ConnectionManager::Pool(pool) => pool.connection(),
            ConnectionManager::Connection(connection) => Ok(connection.duplicate()),
        }
    }

    pub fn transaction_sync<T, E, F>(&self, f: F) -> Result<T, E>
    where
        E: From<RepositoryError>,
        F: FnOnce(ConnectionManager) -> Result<T, E>,
    {
        let connection = self.connection()?;
        let diesel_pooled_connection = connection.diesel_pooled_connection();
        let transaction_manager = diesel_pooled_connection.transaction_manager();

        transaction_manager
            .begin_transaction(diesel_pooled_connection)
            .map_err(|_| RepositoryError::as_db_error("Failed to start tx", ""))?;

        match f(ConnectionManager::Connection(connection.duplicate())) {
            Ok(value) => {
                transaction_manager
                    .commit_transaction(diesel_pooled_connection)
                    .map_err(|_| RepositoryError::as_db_error("Failed to end tx", ""))?;

                Ok(value)
            }
            Err(error) => {
                transaction_manager
                    .rollback_transaction(diesel_pooled_connection)
                    .map_err(|_| RepositoryError::as_db_error("Failed to rollback tx", ""))?;
                Err(error)
            }
        }
    }
}
