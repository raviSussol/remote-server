use std::sync::Arc;

use repository::{RepositoryError, StorageConnection, StorageConnectionManager};

use crate::{
    dashboard::{
        invoice_count::{InvoiceCountService, InvoiceCountServiceTrait},
        stock_expiry_count::{StockExpiryCountServiceTrait, StockExpiryServiceCount},
    },
    location::{LocationService, LocationServiceTrait},
    master_list::{MasterListService, MasterListServiceTrait},
    permission_validation::{ValidationService, ValidationServiceTrait},
    permissions::{PermissionService, PermissionServiceTrait},
    stock_take::{StockTakeService, StockTakeServiceTrait},
    stock_take_line::{StockTakeLineService, StockTakeLineServiceTrait},
};

pub struct ServiceProvider {
    pub connection_manager: StorageConnectionManager,
    pub permission_service: Arc<dyn PermissionServiceTrait>,
    pub validation_service: Box<dyn ValidationServiceTrait>,

    pub location_service: Box<dyn LocationServiceTrait>,
    pub master_list_service: Box<dyn MasterListServiceTrait>,
    pub stock_take_service: Box<dyn StockTakeServiceTrait>,
    pub stock_take_line_service: Box<dyn StockTakeLineServiceTrait>,
    // Dashboard:
    pub invoice_count_service: Box<dyn InvoiceCountServiceTrait>,
    pub stock_expiry_count_service: Box<dyn StockExpiryCountServiceTrait>,
}

pub struct ServiceContext {
    pub connection: StorageConnection,
}

impl ServiceProvider {
    pub fn new(connection_manager: StorageConnectionManager) -> Self {
        let permission_service = Arc::new(PermissionService::new());
        ServiceProvider {
            connection_manager,
            permission_service: permission_service.clone(),
            validation_service: Box::new(ValidationService::new(permission_service)),
            location_service: Box::new(LocationService {}),
            master_list_service: Box::new(MasterListService {}),
            invoice_count_service: Box::new(InvoiceCountService {}),
            stock_expiry_count_service: Box::new(StockExpiryServiceCount {}),
            stock_take_service: Box::new(StockTakeService {}),
            stock_take_line_service: Box::new(StockTakeLineService {}),
        }
    }

    pub fn context(&self) -> Result<ServiceContext, RepositoryError> {
        Ok(ServiceContext {
            connection: self.connection()?,
        })
    }

    pub fn connection(&self) -> Result<StorageConnection, RepositoryError> {
        self.connection_manager.connection()
    }
}
