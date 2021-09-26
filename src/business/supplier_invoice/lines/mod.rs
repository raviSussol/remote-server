use std::convert::TryInto;

use uuid::Uuid;

use crate::{
    database::{
        repository::{InvoiceLineRepository, ItemRepository},
        schema::{InvoiceLineRow, InvoiceRow, StockLineRow},
    },
    server::service::graphql::schema::mutations::supplier_invoice::InsertSupplierInvoiceLineInput,
};

use super::{
    InsertSupplierInvoiceError, InsertSupplierInvoiceLineError as Error,
    InsertSupplierInvoiceLineErrors as LineErrors, Mutations,
};

type Errors = Vec<LineErrors>;
type Input = InsertSupplierInvoiceLineInput;

pub async fn get_insert_line_and_batches(
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
            check_syntax(&lines),
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

fn create_line(
    Input {
        id,
        pack_size,
        batch,
        number_of_packs,
        item_id,
        cost_price_per_pack,
        sell_price_per_pack,
        expiry_date,
    }: Input,
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

fn create_batch(line: &InvoiceLineRow, invoice: &InvoiceRow) -> StockLineRow {
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

impl From<Errors> for InsertSupplierInvoiceError {
    fn from(errors: Errors) -> Self {
        InsertSupplierInvoiceError::InvoiceLineErrors(errors)
    }
}

fn merge_errors(errors: Vec<Errors>) -> Errors {
    let mut result: Errors = Vec::new();
    let errors_flattened: Errors = errors.into_iter().flatten().collect();

    for mut error in errors_flattened.into_iter() {
        let matched = result
            .iter_mut()
            .find(|error_to_match| error_to_match.id == error.id);

        if let Some(matched) = matched {
            matched.errors.append(&mut error.errors);
        } else {
            result.push(error);
        }
    }
    result
}

pub async fn check_exists(
    lines: &Vec<Input>,
    repository: &InvoiceLineRepository,
) -> Result<Errors, InsertSupplierInvoiceError> {
    let ids: Vec<String> = lines.iter().map(|input| input.id.clone()).collect();
    let invoice_lines = repository.find_many_by_id(&ids).await?;

    let result = invoice_lines
        .into_iter()
        .map(|invoice_line| LineErrors {
            id: invoice_line.id,
            errors: vec![Error::InvoiceLineAlreadyExists],
        })
        .collect();
    Ok(result)
}

fn check_syntax(lines: &Vec<Input>) -> Errors {
    let mut result = Vec::new();

    for line in lines {
        let mut errors = Vec::new();

        if line.pack_size < 1 {
            errors.push(Error::PackSizeMustBeAboveOne(line.pack_size))
        }

        if line.sell_price_per_pack < 0.0 {
            errors.push(Error::SellPricePerPackMustBePositive(
                line.sell_price_per_pack,
            ))
        }

        if line.cost_price_per_pack < 0.0 {
            errors.push(Error::CostPricePerPackMustBePositive(
                line.cost_price_per_pack,
            ))
        }

        if errors.len() > 0 {
            result.push(LineErrors {
                id: line.id.clone(),
                errors,
            });
        }
    }

    result
}

async fn check_item_id(
    lines: &Vec<Input>,
    repository: &ItemRepository,
) -> Result<Errors, InsertSupplierInvoiceError> {
    let item_ids: Vec<String> = lines.iter().map(|input| input.item_id.clone()).collect();
    let items = repository.find_many_by_id(&item_ids).await?;

    let item_does_not_exists =
        |line: &&Input| -> bool { !items.iter().any(|item| item.id == line.item_id) };

    let result = lines
        .iter()
        .filter(item_does_not_exists)
        .map(|line| LineErrors {
            id: line.id.clone(),
            errors: vec![Error::ItemIdNotFound(line.item_id.clone())],
        })
        .collect();

    Ok(result)
}
