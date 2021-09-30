use crate::{
    database::schema::{InvoiceLineRow, InvoiceRow, StockLineRow},
    server::service::graphql::schema::mutations::supplier_invoice::InsertSupplierInvoiceLineInput,
};
use std::convert::TryInto;
use uuid::Uuid;

pub fn convert_packsize(pack_size: u32) -> i32 {
    pack_size.try_into().unwrap_or(1)
}

pub fn convert_number_of_packs(number_of_packs: u32) -> i32 {
    number_of_packs.try_into().unwrap_or(0)
}

pub fn create_insert_line(
    _: InsertSupplierInvoiceLineInput,
    invoice: &InvoiceRow,
) -> InvoiceLineRow {
    todo!()
}

pub fn create_batch(line: &InvoiceLineRow, invoice: &InvoiceRow) -> StockLineRow {
    todo!()
}
