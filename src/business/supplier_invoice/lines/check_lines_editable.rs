use crate::{
    business::{FullInvoiceLine, UpsertSupplierInvoiceLineError},
    server::service::graphql::schema::mutations::supplier_invoice::UpsertSupplierInvoiceLineInput,
};

use super::UpsertSupplierInvoiceLineErrors;

pub fn check_update_lines_are_editable(
    update_lines: &Vec<UpsertSupplierInvoiceLineInput>,
    existing_lines: &Vec<FullInvoiceLine>,
) -> Vec<UpsertSupplierInvoiceLineErrors> {
    let is_update_line = |existing_line: &&FullInvoiceLine| {
        update_lines
            .iter()
            .any(|update_line| existing_line.line.id == update_line.id)
    };

    let is_not_editable = |existing_line: &&FullInvoiceLine| match &existing_line.batch {
        Some(stock_line) => {
            stock_line.available_number_of_packs != existing_line.line.number_of_packs
        }
        None => true,
    };

    existing_lines
        .iter()
        .filter(is_update_line)
        .filter(is_not_editable)
        .map(|existing_line| UpsertSupplierInvoiceLineErrors {
            id: existing_line.line.id.clone(),
            errors: vec![UpsertSupplierInvoiceLineError::InvoiceLineIsReserved],
        })
        .collect()
}
