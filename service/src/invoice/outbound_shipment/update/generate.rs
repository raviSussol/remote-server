use chrono::Utc;

use domain::{invoice::InvoiceStatus, name::Name, outbound_shipment::UpdateOutboundShipment};
use repository::{
    schema::{InvoiceLineRow, InvoiceLineRowType, InvoiceRow, StockLineRow},
    InvoiceLineRowRepository, StockLineRowRepository, StorageConnection,
};

use super::UpdateOutboundShipmentError;

pub fn generate(
    existing_invoice: InvoiceRow,
    other_party_option: Option<Name>,
    patch: UpdateOutboundShipment,
    connection: &StorageConnection,
) -> Result<(Option<Vec<StockLineRow>>, InvoiceRow), UpdateOutboundShipmentError> {
    let should_create_batches = should_update_batches(&existing_invoice, &patch);
    let mut update_invoice = existing_invoice;

    set_new_status_datetime(&mut update_invoice, &patch);

    update_invoice.name_id = patch.other_party_id.unwrap_or(update_invoice.name_id);
    update_invoice.comment = patch.comment.or(update_invoice.comment);
    update_invoice.their_reference = patch.their_reference.or(update_invoice.their_reference);
    update_invoice.on_hold = patch.on_hold.unwrap_or(update_invoice.on_hold);
    update_invoice.color = patch.color.or(update_invoice.color);

    if let Some(status) = patch.status {
        update_invoice.status = status.full_status().into()
    }

    if let Some(other_party) = other_party_option {
        update_invoice.name_id = other_party.id;
        update_invoice.name_store_id = other_party.store_id;
    }

    if !should_create_batches {
        Ok((None, update_invoice))
    } else {
        Ok((
            Some(generate_batches(&update_invoice.id, connection)?),
            update_invoice,
        ))
    }
}

pub fn should_update_batches(invoice: &InvoiceRow, patch: &UpdateOutboundShipment) -> bool {
    if let Some(new_invoice_status) = patch.full_status() {
        let invoice_status_index = InvoiceStatus::from(invoice.status.clone()).index();
        let new_invoice_status_index = new_invoice_status.index();

        new_invoice_status_index >= InvoiceStatus::Picked.index()
            && invoice_status_index < new_invoice_status_index
    } else {
        false
    }
}

fn set_new_status_datetime(invoice: &mut InvoiceRow, patch: &UpdateOutboundShipment) {
    if let Some(new_invoice_status) = patch.full_status() {
        let current_datetime = Utc::now().naive_utc();
        let invoice_status_index = InvoiceStatus::from(invoice.status.clone()).index();
        let new_invoice_status_index = new_invoice_status.index();

        let is_status_update = |status: InvoiceStatus| {
            new_invoice_status_index >= status.index()
                && invoice_status_index < new_invoice_status_index
        };

        if is_status_update(InvoiceStatus::Allocated) {
            invoice.allocated_datetime = Some(current_datetime.clone());
        }

        if is_status_update(InvoiceStatus::Picked) {
            invoice.picked_datetime = Some(current_datetime);
        }

        if is_status_update(InvoiceStatus::Shipped) {
            invoice.shipped_datetime = Some(current_datetime);
        }
    }
}

// Returns a list of stock lines that need to be updated
pub fn generate_batches(
    id: &str,
    connection: &StorageConnection,
) -> Result<Vec<StockLineRow>, UpdateOutboundShipmentError> {
    // TODO use InvoiceLineRepository (when r#type is available, use equal_any vs ||)
    let invoice_lines: Vec<InvoiceLineRow> = InvoiceLineRowRepository::new(connection)
        .find_many_by_invoice_id(id)?
        .into_iter()
        .filter(|line| {
            line.r#type == InvoiceLineRowType::StockIn
                || line.r#type == InvoiceLineRowType::StockOut
        })
        .collect();

    let stock_line_ids = invoice_lines
        .iter()
        .filter_map(|line| line.stock_line_id.clone())
        .collect::<Vec<String>>();
    let stock_lines = StockLineRowRepository::new(connection).find_many_by_ids(&stock_line_ids)?;

    let mut result = Vec::new();
    for invoice_line in invoice_lines {
        let stock_line_id = invoice_line.stock_line_id.ok_or(
            UpdateOutboundShipmentError::InvoiceLineHasNoStockLine(invoice_line.id.to_owned()),
        )?;
        let mut stock_line = stock_lines
            .iter()
            .find(|stock_line| stock_line_id == stock_line.id)
            .ok_or(UpdateOutboundShipmentError::InvoiceLineHasNoStockLine(
                invoice_line.id.to_owned(),
            ))?
            .clone();

        stock_line.total_number_of_packs =
            stock_line.total_number_of_packs - invoice_line.number_of_packs;
        result.push(stock_line);
    }
    Ok(result)
}
