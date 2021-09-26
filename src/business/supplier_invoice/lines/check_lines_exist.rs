use crate::{
    business::{
        InsertSupplierInvoiceError, InsertSupplierInvoiceLineError, InsertSupplierInvoiceLineErrors,
    },
    database::repository::{InvoiceLineRepository, RepositoryError},
    server::service::graphql::schema::mutations::supplier_invoice::InsertSupplierInvoiceLineInput,
};

pub async fn check_lines_exist_insert(
    lines: &Vec<InsertSupplierInvoiceLineInput>,
    repository: &InvoiceLineRepository,
) -> Result<Vec<InsertSupplierInvoiceLineErrors>, InsertSupplierInvoiceError> {
    let ids: Vec<String> = lines.iter().map(|input| input.id.clone()).collect();

    let result = check_lines_exist(ids, None, repository).await?;

    let result = result
        .another_invoice
        .into_iter()
        .map(|id| InsertSupplierInvoiceLineErrors {
            id,
            errors: vec![InsertSupplierInvoiceLineError::InvoiceLineAlreadyExists],
        })
        .collect();

    Ok(result)
}

pub struct CheckLinesExistResult {
    in_invoice: Vec<String>,
    not_in_invoice: Vec<String>,
    another_invoice: Vec<String>,
}

pub async fn check_lines_exist(
    ids: Vec<String>,
    ids_in_invoice: Option<Vec<String>>,
    repository: &InvoiceLineRepository,
) -> Result<CheckLinesExistResult, RepositoryError> {
    let (in_invoice, not_in_invoice) = match ids_in_invoice {
        Some(ids_in_invoice) => {
            let SplitIdsResult {
                in_invoice,
                not_in_invoice,
            } = split_ids(ids, ids_in_invoice);
            (in_invoice, not_in_invoice)
        }
        None => (Vec::new(), ids),
    };

    let another_invoice = repository
        .find_many_by_id(&not_in_invoice)
        .await?
        .into_iter()
        .map(|invoice_line| invoice_line.id)
        .collect();

    Ok(CheckLinesExistResult {
        another_invoice,
        in_invoice,
        not_in_invoice,
    })
}

struct SplitIdsResult {
    in_invoice: Vec<String>,
    not_in_invoice: Vec<String>,
}

fn split_ids(to_split: Vec<String>, to_match: Vec<String>) -> SplitIdsResult {
    let mut result = SplitIdsResult {
        in_invoice: Vec::new(),
        not_in_invoice: Vec::new(),
    };
    // Is there iter helper that can return two arrays ?
    for id in to_split.into_iter() {
        let id_is_in_invoice = to_match.iter().any(|id_in_invoice| id_in_invoice == &id);

        if id_is_in_invoice {
            result.in_invoice.push(id)
        } else {
            result.not_in_invoice.push(id)
        }
    }

    result
}
