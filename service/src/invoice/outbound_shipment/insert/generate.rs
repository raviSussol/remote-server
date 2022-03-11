use chrono::Utc;

use repository::Name;
use repository::{
    schema::{InvoiceRow, InvoiceRowStatus, InvoiceRowType, NumberRowType},
    RepositoryError, StorageConnection,
};

use crate::number::next_number;
use crate::user_account::get_default_user_id;

use super::InsertOutboundShipment;

pub fn generate(
    connection: &StorageConnection,
    store_id: &str,
    input: InsertOutboundShipment,
    other_party: Name,
) -> Result<InvoiceRow, RepositoryError> {
    let current_datetime = Utc::now().naive_utc();

    let result = InvoiceRow {
        id: input.id,
        user_id: Some(get_default_user_id()),
        name_id: input.other_party_id,
        r#type: InvoiceRowType::OutboundShipment,
        comment: input.comment,
        their_reference: input.their_reference,
        invoice_number: next_number(connection, &NumberRowType::OutboundShipment, store_id)?,
        name_store_id: other_party.store_id().map(|id| id.to_string()),
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
