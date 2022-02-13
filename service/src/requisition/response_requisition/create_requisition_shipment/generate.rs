use super::{ItemFulFillment, OutError};
use crate::{invoice::check_other_party_id, number::next_number};
use chrono::Utc;
use repository::{
    schema::{
        InvoiceLineRow, InvoiceLineRowType, InvoiceRow, InvoiceRowStatus, InvoiceRowType,
        NumberRowType, RequisitionRow,
    },
    ItemRepository, StorageConnection,
};
use util::uuid::uuid;

pub fn generate(
    connection: &StorageConnection,
    store_id: &str,
    requisition_row: RequisitionRow,
    fullfilments: Vec<ItemFulFillment>,
) -> Result<(InvoiceRow, Vec<InvoiceLineRow>), OutError> {
    let other_party = check_other_party_id(connection, &requisition_row.name_id)?
        .ok_or(OutError::ProblemGettingOtherParty)?;

    let new_invoice = InvoiceRow {
        id: uuid(),
        name_id: requisition_row.name_id,
        name_store_id: other_party.store_id,
        store_id: store_id.to_owned(),
        invoice_number: next_number(connection, &NumberRowType::OutboundShipment, &store_id)?,
        r#type: InvoiceRowType::OutboundShipment,
        status: InvoiceRowStatus::New,
        created_datetime: Utc::now().naive_utc(),
        requisition_id: Some(requisition_row.id),

        // Default
        on_hold: false,
        comment: None,
        their_reference: None,
        allocated_datetime: None,
        picked_datetime: None,
        shipped_datetime: None,
        delivered_datetime: None,
        verified_datetime: None,
        colour: None,
        linked_invoice_id: None,
    };

    let invoice_line_rows = generate_invoice_lines(connection, &new_invoice.id, fullfilments)?;
    Ok((new_invoice, invoice_line_rows))
}

pub fn generate_invoice_lines(
    connection: &StorageConnection,
    invoice_id: &str,
    fullfilments: Vec<ItemFulFillment>,
) -> Result<Vec<InvoiceLineRow>, OutError> {
    let mut invoice_line_rows = vec![];

    for ItemFulFillment { item_id, quantity } in fullfilments.into_iter() {
        let item_row = ItemRepository::new(connection)
            .find_one_by_id(&item_id)?
            .ok_or(OutError::ProblemFindingItem)?;

        invoice_line_rows.push(InvoiceLineRow {
            id: uuid(),
            invoice_id: invoice_id.to_owned(),
            pack_size: 1,
            number_of_packs: quantity,
            item_id,
            item_code: item_row.code,
            item_name: item_row.name,
            r#type: InvoiceLineRowType::UnallocatedStock,

            // Default
            total_before_tax: 0.0,
            total_after_tax: 0.0,
            tax: None,
            note: None,
            location_id: None,
            batch: None,
            expiry_date: None,
            sell_price_per_pack: 0.0,
            cost_price_per_pack: 0.0,
            stock_line_id: None,
        });
    }

    Ok(invoice_line_rows)
}