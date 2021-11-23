use crate::u32_to_i32;
use domain::outbound_shipment::InsertOutboundShipmentLine;
use repository::schema::{InvoiceLineRow, InvoiceRow, InvoiceRowStatus, ItemRow, StockLineRow};

use super::InsertOutboundShipmentLineError;

pub fn generate(
    input: InsertOutboundShipmentLine,
    item_row: ItemRow,
    batch: StockLineRow,
    invoice: InvoiceRow,
) -> Result<(InvoiceLineRow, StockLineRow), InsertOutboundShipmentLineError> {
    let adjust_total_number_of_packs = invoice.status == InvoiceRowStatus::Confirmed;

    let update_batch = generate_batch_update(&input, batch.clone(), adjust_total_number_of_packs);
    let new_line = generate_line(input, item_row, batch);

    Ok((new_line, update_batch))
}

fn generate_batch_update(
    input: &InsertOutboundShipmentLine,
    batch: StockLineRow,
    adjust_total_number_of_packs: bool,
) -> StockLineRow {
    let mut update_batch = batch;

    let reduction = u32_to_i32(input.number_of_packs);

    update_batch.available_number_of_packs = update_batch.available_number_of_packs - reduction;
    if adjust_total_number_of_packs {
        update_batch.total_number_of_packs = update_batch.total_number_of_packs - reduction;
    }

    update_batch
}

fn generate_line(
    InsertOutboundShipmentLine {
        id,
        invoice_id,
        item_id,
        stock_line_id,
        number_of_packs,
    }: InsertOutboundShipmentLine,
    ItemRow {
        name: item_name,
        code: item_code,
        ..
    }: ItemRow,
    StockLineRow {
        sell_price_per_pack,
        cost_price_per_pack,
        pack_size,
        batch,
        expiry_date,
        location_id,
        note,
        ..
    }: StockLineRow,
) -> InvoiceLineRow {
    let total_after_tax = sell_price_per_pack * number_of_packs as f64;

    InvoiceLineRow {
        id,
        invoice_id,
        item_id,
        location_id,
        pack_size,
        batch,
        expiry_date,
        sell_price_per_pack,
        cost_price_per_pack,
        number_of_packs: u32_to_i32(number_of_packs),
        item_name,
        item_code,
        stock_line_id: Some(stock_line_id),
        total_after_tax,
        note,
    }
}
