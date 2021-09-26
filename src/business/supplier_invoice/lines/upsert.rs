use check_item_id::check_item_id_upsert;

use crate::{
    business::{
        add_inserts, add_updates, check_item_id, check_syntax_upsert,
        check_update_lines_are_editable, get_inserts_from_upsert, get_lines_by_type, merge_errors,
        Delete, FullInvoiceLine, GroupedUpsertLines, InsertsFromUpsert, Mutations,
        UpdateSupplierInvoiceError,
    },
    database::{
        repository::{InvoiceLineRepository, ItemRepository},
        schema::{InvoiceLineRow, InvoiceRow, StockLineRow},
    },
    server::service::graphql::schema::mutations::supplier_invoice::UpsertSupplierInvoiceLineInput as Input,
};

pub async fn get_upsert_lines_and_batches(
    lines: Option<Vec<Input>>,
    invoice_line_repository: &InvoiceLineRepository,
    item_respository: &ItemRepository,
    existing_invoice: &InvoiceRow,
    existing_invoice_lines: Vec<FullInvoiceLine>,
) -> Result<(Mutations<InvoiceLineRow>, Mutations<StockLineRow>), UpdateSupplierInvoiceError> {
    let mut line_mutations = Mutations::new();
    let mut batch_mutations = Mutations::new();

    if let Some(lines) = lines {
        let GroupedUpsertLines {
            inserts,
            updates,
            deletes,
            errors: errors_in_grouping,
        } = get_lines_by_type(lines, &existing_invoice_lines, invoice_line_repository).await?;

        let mut all_errors = vec![
            errors_in_grouping,
            check_syntax_upsert(&inserts),
            check_syntax_upsert(&updates),
            check_item_id_upsert(&inserts, item_respository).await?,
            check_item_id_upsert(&updates, item_respository).await?,
            check_update_lines_are_editable(&updates, &existing_invoice_lines),
        ];

        let InsertsFromUpsert {
            inserts,
            errors: inserts_syntax_errors,
        } = get_inserts_from_upsert(inserts);

        all_errors.push(inserts_syntax_errors);

        let all_errors = merge_errors(all_errors);

        if all_errors.len() > 0 {
            return Err(all_errors.into());
        }

        add_inserts(
            inserts,
            existing_invoice,
            &mut line_mutations,
            &mut batch_mutations,
        );

        add_deletes(deletes, &mut line_mutations, &mut batch_mutations);

        add_updates(
            updates,
            existing_invoice,
            existing_invoice_lines,
            &mut line_mutations,
            &mut batch_mutations,
        );
    }

    Ok((line_mutations, batch_mutations))
}

pub fn add_deletes(
    deletes: Vec<Delete>,
    line_mutations: &mut Mutations<InvoiceLineRow>,
    batch_mutations: &mut Mutations<StockLineRow>,
) {
    for delete in deletes.into_iter() {
        line_mutations.add_delete(delete.line_id);

        if let Some(stock_line_id) = delete.stock_line_id {
            batch_mutations.add_delete(stock_line_id);
        }
    }
}
