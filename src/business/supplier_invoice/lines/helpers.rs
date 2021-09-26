use crate::{
    database::schema::{InvoiceLineRow, InvoiceRow, StockLineRow},
    server::service::graphql::schema::mutations::supplier_invoice::InsertSupplierInvoiceLineInput,
};
use std::convert::TryInto;
use uuid::Uuid;

pub fn create_insert_line(
    InsertSupplierInvoiceLineInput {
        id,
        pack_size,
        batch,
        number_of_packs,
        item_id,
        cost_price_per_pack,
        sell_price_per_pack,
        expiry_date,
    }: InsertSupplierInvoiceLineInput,
    invoice: &InvoiceRow,
) -> InvoiceLineRow {
    let pack_size = pack_size.try_into().unwrap_or(1);
    let number_of_packs = number_of_packs.try_into().unwrap_or(0);

    let total_after_tax = cost_price_per_pack * pack_size as f64 * number_of_packs as f64;

    InvoiceLineRow {
        id,
        invoice_id: invoice.id.clone(),
        item_id,
        stock_line_id: None,
        batch,
        expiry_date,
        pack_size,
        number_of_packs,
        cost_price_per_pack,
        sell_price_per_pack,
        total_after_tax,
    }
}

pub fn create_batch(line: &InvoiceLineRow, invoice: &InvoiceRow) -> StockLineRow {
    StockLineRow {
        id: Uuid::new_v4().to_string(),
        item_id: line.item_id.clone(),
        store_id: invoice.store_id.to_string(),
        batch: line.batch.clone(),
        pack_size: line.pack_size,
        cost_price_per_pack: line.cost_price_per_pack,
        sell_price_per_pack: line.sell_price_per_pack,
        available_number_of_packs: line.number_of_packs,
        total_number_of_packs: line.number_of_packs,
        expiry_date: line.expiry_date.clone(),
    }
}
