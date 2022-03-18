use self::{
    query::{get_requisition, get_requisition_by_number, get_requisitions},
    request_requisition::{
        add_from_master_list, batch_request_requisition, delete_request_requisition,
        insert_request_requisition, update_request_requisition, use_suggested_quantity,
        AddFromMasterList, AddFromMasterListError, BatchRequestRequisition,
        BatchRequestRequisitionResult, DeleteRequestRequisition, DeleteRequestRequisitionError,
        InsertRequestRequisition, InsertRequestRequisitionError, UpdateRequestRequisition,
        UpdateRequestRequisitionError, UseSuggestedQuantity, UseSuggestedQuantityError,
    },
    requisition_supply_status::{get_requisitions_supply_statuses, RequisitionLineSupplyStatus},
    response_requisition::{
        create_requisition_shipment, supply_requested_quantity, update_response_requisition,
        CreateRequisitionShipment, CreateRequisitionShipmentError, SupplyRequestedQuantity,
        SupplyRequestedQuantityError, UpdateResponseRequisition, UpdateResponseRequisitionError,
    },
};

use super::{ListError, ListResult};
use crate::service_provider::ServiceContext;
use repository::PaginationOption;
use repository::{
    schema::RequisitionRowType, Invoice, RepositoryError, Requisition, RequisitionFilter,
    RequisitionLine, RequisitionSort,
};

pub mod common;
pub mod query;
pub mod request_requisition;
pub mod requisition_supply_status;
pub mod response_requisition;

pub trait RequisitionServiceTrait: Sync + Send {
    fn get_requisitions(
        &self,
        ctx: &ServiceContext,
        store_id_option: Option<&str>,
        pagination: Option<PaginationOption>,
        filter: Option<RequisitionFilter>,
        sort: Option<RequisitionSort>,
    ) -> Result<ListResult<Requisition>, ListError> {
        get_requisitions(ctx, store_id_option, pagination, filter, sort)
    }

    fn get_requisition(
        &self,
        ctx: &ServiceContext,
        store_id_option: Option<&str>,
        id: &str,
    ) -> Result<Option<Requisition>, RepositoryError> {
        get_requisition(ctx, store_id_option, id)
    }

    fn get_requisition_by_number(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        requisition_number: u32,
        r#type: RequisitionRowType,
    ) -> Result<Option<Requisition>, RepositoryError> {
        get_requisition_by_number(ctx, store_id, requisition_number, r#type)
    }

    fn get_requisitions_supply_status(
        &self,
        ctx: &ServiceContext,
        requisition_ids: Vec<String>,
    ) -> Result<Vec<RequisitionLineSupplyStatus>, RepositoryError> {
        get_requisitions_supply_statuses(&ctx.connection, requisition_ids)
    }

    fn insert_request_requisition(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        user_id: &str,
        input: InsertRequestRequisition,
    ) -> Result<Requisition, InsertRequestRequisitionError> {
        insert_request_requisition(ctx, store_id, user_id, input)
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

    fn use_suggested_quantity(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: UseSuggestedQuantity,
    ) -> Result<Vec<RequisitionLine>, UseSuggestedQuantityError> {
        use_suggested_quantity(ctx, store_id, input)
    }

    fn add_from_master_list(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: AddFromMasterList,
    ) -> Result<Vec<RequisitionLine>, AddFromMasterListError> {
        add_from_master_list(ctx, store_id, input)
    }

    fn update_response_requisition(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        user_id: &str,
        input: UpdateResponseRequisition,
    ) -> Result<Requisition, UpdateResponseRequisitionError> {
        update_response_requisition(ctx, store_id, user_id, input)
    }

    fn supply_requested_quantity(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        user_id: &str,
        input: SupplyRequestedQuantity,
    ) -> Result<Vec<RequisitionLine>, SupplyRequestedQuantityError> {
        supply_requested_quantity(ctx, store_id, user_id, input)
    }

    fn create_requisition_shipment(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        user_id: &str,
        input: CreateRequisitionShipment,
    ) -> Result<Invoice, CreateRequisitionShipmentError> {
        create_requisition_shipment(ctx, store_id, user_id, input)
    }

    fn batch_request_requisition(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        user_id: &str,
        input: BatchRequestRequisition,
    ) -> Result<BatchRequestRequisitionResult, RepositoryError> {
        batch_request_requisition(ctx, store_id, user_id, input)
    }
}

pub struct RequisitionService {}
impl RequisitionServiceTrait for RequisitionService {}
