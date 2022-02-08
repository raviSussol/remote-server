use self::{
    query::{get_requisition_by_number, get_requisitions},
    request_requisition::{
        add_from_master_list, delete_request_requisition, insert_request_requisition,
        update_request_requisition, use_calculated_quantity, AddFromMasterList,
        AddFromMasterListError, DeleteRequestRequisition, DeleteRequestRequisitionError,
        InsertRequestRequisition, InsertRequestRequisitionError, UpdateRequestRequisition,
        UpdateRequestRequisitionError, UseCalculatedQuantity, UseCalculatedQuantityError,
    },
    response_requisition::{
        create_requisition_shipment, supply_requested_quantity, update_response_requisition,
        CreateRequisitionShipment, CreateRequisitionShipmentError, SupplyRequestedQuantity,
        SupplyRequestedQuantityError, UpdateResponseRequisition, UpdateResponseRequisitionError,
    },
};

use super::{ListError, ListResult};
use crate::service_provider::ServiceContext;
use domain::{invoice::Invoice, PaginationOption};
use repository::{
    schema::RequisitionRowType, RepositoryError, Requisition, RequisitionFilter, RequisitionLine,
    RequisitionSort,
};

pub mod common;
pub mod query;
pub mod request_requisition;
pub mod response_requisition;

pub trait RequisitionServiceTrait: Sync + Send {
    fn insert_request_requisition(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: InsertRequestRequisition,
    ) -> Result<Requisition, InsertRequestRequisitionError> {
        insert_request_requisition(ctx, store_id, input)
    }

    fn update_request_requisition(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: UpdateRequestRequisition,
    ) -> Result<Requisition, UpdateRequestRequisitionError> {
        update_request_requisition(ctx, store_id, input)
    }

    fn delete_request_requisition(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: DeleteRequestRequisition,
    ) -> Result<String, DeleteRequestRequisitionError> {
        delete_request_requisition(ctx, store_id, input)
    }

    fn use_calculated_quantity(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: UseCalculatedQuantity,
    ) -> Result<Vec<RequisitionLine>, UseCalculatedQuantityError> {
        use_calculated_quantity(ctx, store_id, input)
    }

    fn update_response_requisition(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: UpdateResponseRequisition,
    ) -> Result<Requisition, UpdateResponseRequisitionError> {
        update_response_requisition(ctx, store_id, input)
    }

    fn supply_requested_quantity(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: SupplyRequestedQuantity,
    ) -> Result<Vec<RequisitionLine>, SupplyRequestedQuantityError> {
        supply_requested_quantity(ctx, store_id, input)
    }

    fn create_requisition_shipment(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: CreateRequisitionShipment,
    ) -> Result<Invoice, CreateRequisitionShipmentError> {
        create_requisition_shipment(ctx, store_id, input)
    }
}

pub struct RequisitionService {}
impl RequisitionServiceTrait for RequisitionService {}
