use chrono::Utc;

use repository::{
    schema::{InvoiceRow, InvoiceRowStatus, InvoiceRowType, NumberRowType},
    Name, RepositoryError, StorageConnection,
};

use crate::number::next_number;

use super::InsertInboundShipment;

pub fn generate(
    connection: &StorageConnection,
    store_id: &str,
    user_id: &str,
    InsertInboundShipment {
        id,
        other_party_id,
        on_hold,
        comment,
        their_reference,
        colour,
    }: InsertInboundShipment,
    other_party: Name,
) -> Result<InvoiceRow, RepositoryError> {
    let current_datetime = Utc::now().naive_utc();

    let result = InvoiceRow {
        id,
        user_id: Some(user_id.to_string()),
        name_id: other_party_id,
        name_store_id: other_party.store_id().map(|id| id.to_string()),
        r#type: InvoiceRowType::InboundShipment,
        comment,
        their_reference,
        invoice_number: next_number(connection, &NumberRowType::InboundShipment, store_id)?,
        store_id: store_id.to_string(),
        created_datetime: current_datetime,
        status: InvoiceRowStatus::New,
        on_hold: on_hold.unwrap_or(false),
        colour,
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
