use crate::{
    business::{
        convert_number_of_packs, convert_packsize, create_batch, FullInvoiceLine, Mutations,
    },
    database::schema::{InvoiceLineRow, InvoiceRow, InvoiceRowStatus, StockLineRow},
    server::service::graphql::schema::mutations::supplier_invoice::UpsertSupplierInvoiceLineInput as Input,
};

pub fn add_updates(
    mut updates: Vec<Input>,
    invoice: &InvoiceRow,
    invoice_lines: Vec<FullInvoiceLine>,
    line_mutations: &mut Mutations<InvoiceLineRow>,
    batch_mutations: &mut Mutations<StockLineRow>,
) {
    let mut get_update_element = |line: &FullInvoiceLine| match updates
        .iter()
        .position(|update_line| line.line.id == update_line.id)
    {
        Some(position) => Some(updates.remove(position)),
        None => None,
    };

    let lines_with_updates: Vec<(Option<Input>, FullInvoiceLine)> = invoice_lines
        .into_iter()
        .map(|line| (get_update_element(&line), line))
        .collect();

    for (update_line, line) in lines_with_updates {
        if let Some(update) = update_line {
            let mut new_line = get_updated_invoice_line(line.line, update);

            if invoice.status != InvoiceRowStatus::Draft {
                add_batch(line.batch, &mut new_line, invoice, batch_mutations);
            }
            line_mutations.add_update(new_line);
        }
    }
}

pub fn add_batch(
    stock_line: Option<StockLineRow>,
    line: &mut InvoiceLineRow,
    invoice: &InvoiceRow,
    batch_mutations: &mut Mutations<StockLineRow>,
) {
    let mut new_stock_line = create_batch(line, invoice);

    match stock_line {
        Some(existing_batch) => {
            new_stock_line.id = existing_batch.id.clone();
            batch_mutations.add_update(new_stock_line);
        }
        None => {
            line.stock_line_id = Some(new_stock_line.id.clone());
            batch_mutations.add_insert(new_stock_line);
        }
    }
}

fn get_updated_invoice_line(mut line: InvoiceLineRow, update_line: Input) -> InvoiceLineRow {
    let Input {
        id: _,
        item_id,
        pack_size,
        batch,
        cost_price_per_pack,
        sell_price_per_pack,
        expiry_date,
        number_of_packs,
    } = update_line;

    todo!()
}
