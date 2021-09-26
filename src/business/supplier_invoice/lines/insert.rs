use crate::{
    business::{
        check_item_id_insert, check_lines_exist_insert, check_syntax_insert, create_batch,
        create_insert_line, merge_errors, InsertSupplierInvoiceError, Mutations,
    },
    database::{
        repository::{InvoiceLineRepository, ItemRepository},
        schema::{InvoiceLineRow, InvoiceRow, StockLineRow},
    },
    server::service::graphql::schema::mutations::supplier_invoice::InsertSupplierInvoiceLineInput as Input,
};

pub async fn get_insert_lines_and_batches(
    lines: Option<Vec<Input>>,
    invoice_line_repository: &InvoiceLineRepository,
    item_respository: &ItemRepository,
    invoice: &InvoiceRow,
) -> Result<(Mutations<InvoiceLineRow>, Mutations<StockLineRow>), InsertSupplierInvoiceError> {
    let mut new_lines = Mutations::new();
    let mut new_batches = Mutations::new();

    if let Some(lines) = lines {
        let all_errors = merge_errors(vec![
            check_lines_exist_insert(&lines, invoice_line_repository).await?,
            check_syntax_insert(&lines),
            check_item_id_insert(&lines, item_respository).await?,
        ]);

        if all_errors.len() > 0 {
            return Err(all_errors.into());
        }

        for line in lines {
            let mut new_line = create_insert_line(line, invoice);
            let new_batch = create_batch(&new_line, invoice);
            new_line.stock_line_id = Some(new_batch.id.clone());

            new_lines.add_insert(new_line);
            new_batches.add_insert(new_batch);
        }
    }

    Ok((new_lines, new_batches))
}
