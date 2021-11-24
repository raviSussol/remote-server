use repository::{RepositoryError, StorageConnection, StorageConnectionManager, TransactionError};

use crate::location::{LocationService, LocationServiceQuery};

pub struct ServiceInstances {
    pub location_service: Box<dyn LocationServiceQuery>,
}

impl ServiceInstances {
    pub fn new() -> Self {
        ServiceInstances {
            location_service: Box::new(LocationService {}),
        }
    }
}

pub enum ServiceError<E> {
    RepositoryError(RepositoryError),
    Inner(E),
}

pub struct ServicesProvider {
    connection_manager: StorageConnectionManager,
    pub services_instances: ServiceInstances,
}

impl ServicesProvider {
    pub fn new(connection_manager: StorageConnectionManager) -> Self {
        ServicesProvider {
            connection_manager,
            services_instances: ServiceInstances::new(),
        }
    }

    pub fn set_service_instances(mut self, services_instances: ServiceInstances) -> Self {
        self.services_instances = services_instances;
        self
    }

    pub fn services<'a>(&'a self) -> Result<Services<'a>, RepositoryError> {
        let services = Services {
            services_instances: &self.services_instances,
            connection: ServicesConnection::Instance(self.connection_manager.connection()?),
        };
        Ok(services)
    }

    pub fn services_tx<T, E, F>(&self, f: F) -> Result<T, ServiceError<E>>
    where
        F: FnOnce(&Services) -> Result<T, E>,
    {
        let connection = &self.connection_manager.connection()?;
        let result = connection.transaction_sync(|connection_tx| {
            let services_tx = Services {
                services_instances: &self.services_instances,
                connection: ServicesConnection::Reference(&connection_tx),
            };
            f(&services_tx)
        });

        result.map_err(|error| match error {
            TransactionError::Transaction { msg } => {
                ServiceError::RepositoryError(RepositoryError::as_db_error(&msg, ""))
            }
            TransactionError::Inner(error) => ServiceError::Inner(error),
        })
    }
}

pub enum ServicesConnection<'a> {
    Instance(StorageConnection),
    Reference(&'a StorageConnection),
}
pub struct Services<'a> {
    pub services_instances: &'a ServiceInstances,
    pub connection: ServicesConnection<'a>,
}

impl<'a> Services<'a> {
    pub fn get_connection(&self) -> &StorageConnection {
        match &self.connection {
            ServicesConnection::Instance(instance) => instance,
            ServicesConnection::Reference(reference) => reference,
        }
    }
}

impl<E> From<RepositoryError> for ServiceError<E> {
    fn from(error: RepositoryError) -> Self {
        ServiceError::RepositoryError(error)
    }
}
