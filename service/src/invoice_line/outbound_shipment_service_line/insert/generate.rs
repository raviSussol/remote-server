use repository::schema::{InvoiceLineRow, InvoiceLineRowType, ItemRow};

use super::{InsertOutboundShipmentServiceLine, InsertOutboundShipmentServiceLineError};

pub fn generate(
    InsertOutboundShipmentServiceLine {
        id,
        invoice_id,
        item_id: _,
        name,
        total_before_tax,
        total_after_tax,
        tax,
        note,
    }: InsertOutboundShipmentServiceLine,
    item: ItemRow,
) -> Result<InvoiceLineRow, InsertOutboundShipmentServiceLineError> {
    Ok(InvoiceLineRow {
        id,
        invoice_id,
        total_before_tax,
        total_after_tax,
        tax,
        note,
        item_code: item.code,
        item_id: item.id,
        item_name: name.unwrap_or(item.name),
        r#type: InvoiceLineRowType::Service,
        // Default
        stock_line_id: None,
        location_id: None,
        batch: None,
        expiry_date: None,
        pack_size: 0,
        cost_price_per_pack: 0.0,
        sell_price_per_pack: 0.0,
        number_of_packs: 0,
    })
}
