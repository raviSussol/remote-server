use crate::{
    business::InsertSupplierInvoiceError,
    database::{
        repository::{InvoiceLineRepository, ItemRepository},
        schema::{InvoiceLineRow, InvoiceRow},
    },
    server::service::graphql::schema::mutations::supplier_invoice::InsertSupplierInvoiceLineInput as Input,
};

pub async fn get_upsert_lines_and_batches(
    lines: Option<Vec<Input>>,
    invoice_line_repository: &InvoiceLineRepository,
    item_respository: &ItemRepository,
    invoice: &InvoiceRow,
) -> Result<(Mutations<InvoiceLineRow>, Mutations<StockLineRow>), InsertSupplierInvoiceError> {
    let mut new_lines = Mutations::new();
    let mut new_batches = Mutations::new();

    if let Some(lines) = lines {
        let all_errors = merge_errors(vec![
            check_exists(&lines, invoice_line_repository).await?,
            check_syntax_insert(&lines),
            check_item_id(&lines, item_respository).await?,
        ]);

        if all_errors.len() > 0 {
            return Err(all_errors.into());
        }

        for line in lines {
            let mut new_line = create_line(line, invoice);
            let new_batch = create_batch(&new_line, invoice);
            new_line.stock_line_id = Some(new_batch.id.clone());

            new_lines.add_insert(new_line);
            new_batches.add_insert(new_batch);
        }
    }

    Ok((new_lines, new_batches))
}

// validate line and add to errors
// if status is not draft add to available and total stock of batch and add batch

// And for update ?

// filter by inserts and update
// filter out missing

// for udpate check x and y
