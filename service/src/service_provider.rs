use repository::storage_connection_example::{ConnectionManager, ConnectionPool};

use crate::location::{query::LocationQueryService, LocationQueryServiceTrait};

pub trait ServiceFactoryTrait: Sync + Send {
    fn location_service<'a>(
        &self,
        connection: ConnectionManager<'a>,
    ) -> Box<dyn LocationQueryServiceTrait + 'a> {
        Box::new(LocationQueryService { connection })
    }
}

pub struct ServiceFactory;
impl ServiceFactoryTrait for ServiceFactory {}

pub struct ServiceProvider {
    connection_pool: ConnectionPool,
    pub service_factory: Box<dyn ServiceFactoryTrait>,
}

impl<'a> ServiceProvider {
    pub fn new(connection_pool: ConnectionPool) -> Self {
        ServiceProvider {
            connection_pool,
            service_factory: Box::new(ServiceFactory {}),
        }
    }

    pub fn set_service_factory(&mut self, service_factory: Box<dyn ServiceFactoryTrait>) {
        self.service_factory = service_factory;
    }

    pub fn connection_manager(&self) -> ConnectionManager<'a> {
        self.connection_pool.connection_manager()
    }

    pub fn location_service(&self) -> Box<dyn LocationQueryServiceTrait> {
        self.service_factory
            .location_service(self.connection_manager())
    }
}
