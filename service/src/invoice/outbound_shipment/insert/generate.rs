use chrono::Utc;

use domain::{invoice::InvoiceType, name::Name, outbound_shipment::InsertOutboundShipment};
use repository::{
    schema::{InvoiceRow, InvoiceRowStatus, NumberRowType},
    RepositoryError, StorageConnection,
};

use crate::number::next_number;

pub fn generate(
    connection: &StorageConnection,
    store_id: &str,
    input: InsertOutboundShipment,
    other_party: Name,
) -> Result<InvoiceRow, RepositoryError> {
    let current_datetime = Utc::now().naive_utc();

    let result = InvoiceRow {
        id: input.id,
        name_id: input.other_party_id,
        r#type: InvoiceType::OutboundShipment.into(),
        comment: input.comment,
        their_reference: input.their_reference,
        invoice_number: next_number(connection, &NumberRowType::OutboundShipment, store_id)?,
        name_store_id: other_party.store_id,
        store_id: store_id.to_string(),
        created_datetime: current_datetime,
        status: InvoiceRowStatus::New,
        on_hold: input.on_hold.unwrap_or(false),
        colour: input.colour,
        allocated_datetime: None,
        picked_datetime: None,
        shipped_datetime: None,
        delivered_datetime: None,
        verified_datetime: None,
        linked_invoice_id: None,
        requisition_id: None,
    };

    Ok(result)
}
