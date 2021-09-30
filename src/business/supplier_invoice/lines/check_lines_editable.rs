use crate::{
    business::{FullInvoiceLine, UpsertSupplierInvoiceLineError},
    server::service::graphql::schema::mutations::supplier_invoice::UpsertSupplierInvoiceLineInput,
};

use super::UpsertSupplierInvoiceLineErrors;

pub fn check_update_lines_are_editable(
    update_lines: &Vec<UpsertSupplierInvoiceLineInput>,
    existing_lines: &Vec<FullInvoiceLine>,
) -> Vec<UpsertSupplierInvoiceLineErrors> {
    todo!()
}
