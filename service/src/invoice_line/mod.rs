pub mod validate;

use repository::InvoiceLine;
use repository::InvoiceLineFilter;
use repository::RepositoryError;

use crate::service_provider::ServiceContext;

pub mod query;
use self::query::*;

pub mod inbound_shipment_line;
use self::inbound_shipment_line::*;

pub mod outbound_shipment_line;
use self::outbound_shipment_line::*;

pub mod outbound_shipment_service_line;
use self::outbound_shipment_service_line::*;

pub mod outbound_shipment_unallocated_line;
use self::outbound_shipment_unallocated_line::*;

pub trait InvoiceLineServiceTrait: Sync + Send {
    fn get_invoice_line(
        &self,
        ctx: &ServiceContext,
        id: &str,
    ) -> Result<Option<InvoiceLine>, RepositoryError> {
        get_invoice_line(ctx, id)
    }

    fn get_invoice_lines(
        &self,
        ctx: &ServiceContext,
        filter: Option<InvoiceLineFilter>,
    ) -> Result<Vec<InvoiceLine>, RepositoryError> {
        get_invoice_lines(ctx, filter)
    }

    fn insert_outbound_shipment_line(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: InsertOutboundShipmentLine,
    ) -> Result<InvoiceLine, InsertOutboundShipmentLineError> {
        insert_outbound_shipment_line(ctx, store_id, input)
    }

    fn update_outbound_shipment_line(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: UpdateOutboundShipmentLine,
    ) -> Result<InvoiceLine, UpdateOutboundShipmentLineError> {
        update_outbound_shipment_line(ctx, store_id, input)
    }

    fn delete_outbound_shipment_line(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: DeleteOutboundShipmentLine,
    ) -> Result<String, DeleteOutboundShipmentLineError> {
        delete_outbound_shipment_line(ctx, store_id, input)
    }

    fn insert_inbound_shipment_line(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: InsertInboundShipmentLine,
    ) -> Result<InvoiceLine, InsertInboundShipmentLineError> {
        insert_inbound_shipment_line(ctx, store_id, input)
    }

    fn update_inbound_shipment_line(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: UpdateInboundShipmentLine,
    ) -> Result<InvoiceLine, UpdateInboundShipmentLineError> {
        update_inbound_shipment_line(ctx, store_id, input)
    }

    fn delete_inbound_shipment_line(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: DeleteInboundShipmentLine,
    ) -> Result<String, DeleteInboundShipmentLineError> {
        delete_inbound_shipment_line(ctx, store_id, input)
    }

    fn insert_outbound_shipment_service_line(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: InsertOutboundShipmentServiceLine,
    ) -> Result<InvoiceLine, InsertOutboundShipmentServiceLineError> {
        insert_outbound_shipment_service_line(ctx, store_id, input)
    }

    fn update_outbound_shipment_service_line(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: UpdateOutboundShipmentServiceLine,
    ) -> Result<InvoiceLine, UpdateOutboundShipmentServiceLineError> {
        update_outbound_shipment_service_line(ctx, store_id, input)
    }

    fn delete_outbound_shipment_service_line(
        &self,
        ctx: &ServiceContext,
        store_id: &str,
        input: DeleteOutboundShipmentLine,
    ) -> Result<String, DeleteOutboundShipmentServiceLineError> {
        delete_outbound_shipment_service_line(ctx, store_id, input)
    }

    fn insert_outbound_shipment_unallocated_line(
        &self,
        ctx: &ServiceContext,
        input: InsertOutboundShipmentUnallocatedLine,
    ) -> Result<InvoiceLine, InsertOutboundShipmentUnallocatedLineError> {
        insert_outbound_shipment_unallocated_line(ctx, input)
    }

    fn update_outbound_shipment_unallocated_line(
        &self,
        ctx: &ServiceContext,
        input: UpdateOutboundShipmentUnallocatedLine,
    ) -> Result<InvoiceLine, UpdateOutboundShipmentUnallocatedLineError> {
        update_outbound_shipment_unallocated_line(ctx, input)
    }

    fn delete_outbound_shipment_unallocated_line(
        &self,
        ctx: &ServiceContext,
        input: DeleteOutboundShipmentUnallocatedLine,
    ) -> Result<String, DeleteOutboundShipmentUnallocatedLineError> {
        delete_outbound_shipment_unallocated_line(ctx, input)
    }
}

pub struct InvoiceLineService {}
impl InvoiceLineServiceTrait for InvoiceLineService {}

pub struct ShipmentTaxUpdate {
    /// Set or unset the tax value
    pub percentage: Option<f64>,
}
