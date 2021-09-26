use crate::{
    business::{
        FullInvoiceLine, InsertSupplierInvoiceError, InsertSupplierInvoiceLineError,
        InsertSupplierInvoiceLineErrors, UpdateSupplierInvoiceError,
        UpsertSupplierInvoiceLineError, UpsertSupplierInvoiceLineErrors,
    },
    database::repository::{InvoiceLineRepository, RepositoryError},
    server::service::graphql::schema::mutations::supplier_invoice::{
        InsertSupplierInvoiceLineInput, UpsertSupplierInvoiceLineInput,
    },
};

pub async fn check_lines_ids_insert(
    lines: &Vec<InsertSupplierInvoiceLineInput>,
    repository: &InvoiceLineRepository,
) -> Result<Vec<InsertSupplierInvoiceLineErrors>, InsertSupplierInvoiceError> {
    let ids: Vec<String> = lines.iter().map(|input| input.id.clone()).collect();

    let result = group_line_ids(ids, None, repository).await?;

    let result = result
        .in_another_invoice
        .into_iter()
        .map(|id| InsertSupplierInvoiceLineErrors {
            id,
            errors: vec![InsertSupplierInvoiceLineError::InvoiceLineAlreadyExists],
        })
        .collect();

    Ok(result)
}

type UsertLine = UpsertSupplierInvoiceLineInput;
type UpsertVec = Vec<UpsertSupplierInvoiceLineInput>;

pub struct Delete {
    pub line_id: String,
    pub stock_line_id: Option<String>,
}
pub struct GroupedUpsertLines {
    pub inserts: UpsertVec,
    pub updates: UpsertVec,
    pub deletes: Vec<Delete>,
    pub errors: Vec<UpsertSupplierInvoiceLineErrors>,
}

pub async fn get_lines_by_type(
    input_lines: UpsertVec,
    existing_lines: &Vec<FullInvoiceLine>,
    repository: &InvoiceLineRepository,
) -> Result<GroupedUpsertLines, UpdateSupplierInvoiceError> {
    let input_ids: Vec<String> = input_lines.iter().map(|input| input.id.clone()).collect();
    let existing_ids: Vec<String> = existing_lines
        .iter()
        .map(|existing| existing.line.id.clone())
        .collect();

    let grouped = group_line_ids(input_ids, Some(existing_ids), repository).await?;

    let is_insert = |line: &UsertLine| {
        grouped
            .in_input_not_in_invoice
            .iter()
            .any(|id| &line.id == id)
    };
    let (inserts, input_lines): (UpsertVec, UpsertVec) =
        input_lines.into_iter().partition(is_insert);

    let is_update = |line: &UsertLine| grouped.in_input_in_invoice.iter().any(|id| &line.id == id);
    let (updates, _): (UpsertVec, UpsertVec) = input_lines.into_iter().partition(is_update);

    let is_delete = |existing_line: &&FullInvoiceLine| {
        grouped
            .not_in_input_in_invoice
            .iter()
            .any(|id| &existing_line.line.id == id)
    };

    let deletes = existing_lines
        .iter()
        .filter(is_delete)
        .map(Delete::from)
        .collect();

    Ok(GroupedUpsertLines {
        inserts,
        updates,
        deletes,
        errors: grouped
            .in_another_invoice
            .into_iter()
            .map(|id| UpsertSupplierInvoiceLineErrors {
                id,
                errors: vec![UpsertSupplierInvoiceLineError::InvoiceLineBelongsToAnotherInvoice],
            })
            .collect(),
    })
}

impl From<&FullInvoiceLine> for Delete {
    fn from(line: &FullInvoiceLine) -> Self {
        Delete {
            line_id: line.line.id.clone(),
            stock_line_id: match &line.batch {
                Some(batch) => Some(batch.id.clone()),
                None => None,
            },
        }
    }
}

pub struct GroupedLineIds {
    in_input_in_invoice: Vec<String>,
    in_input_not_in_invoice: Vec<String>,
    not_in_input_in_invoice: Vec<String>,
    in_another_invoice: Vec<String>,
}

pub async fn group_line_ids(
    ids: Vec<String>,
    ids_in_invoice: Option<Vec<String>>,
    repository: &InvoiceLineRepository,
) -> Result<GroupedLineIds, RepositoryError> {
    let (in_input_in_invoice, in_input_not_in_invoice, not_in_input_in_invoice) =
        match ids_in_invoice {
            Some(ids_in_invoice) => {
                let split = split_ids(ids, ids_in_invoice);
                (
                    split.in_input_in_invoice,
                    split.in_input_not_in_invoice,
                    split.not_in_input_in_invoice,
                )
            }
            None => (Vec::new(), ids, Vec::new()),
        };

    let in_another_invoice = repository
        .find_many_by_id(&in_input_not_in_invoice)
        .await?
        .into_iter()
        .map(|invoice_line| invoice_line.id)
        .collect();

    Ok(GroupedLineIds {
        in_another_invoice,
        in_input_in_invoice,
        in_input_not_in_invoice,
        not_in_input_in_invoice,
    })
}

struct SplitIdsResult {
    in_input_in_invoice: Vec<String>,
    in_input_not_in_invoice: Vec<String>,
    not_in_input_in_invoice: Vec<String>,
}

fn split_ids(input: Vec<String>, existing: Vec<String>) -> SplitIdsResult {
    let is_in_input = |id: &String| input.iter().any(|id_in_input| id_in_input == id);

    let (not_in_input_in_invoice, existing): (Vec<String>, Vec<String>) =
        existing.into_iter().partition(is_in_input);

    let is_in_existing = |id: &String| existing.iter().any(|id_in_existing| id_in_existing == id);

    let (in_input_in_invoice, in_input_not_in_invoice): (Vec<String>, Vec<String>) =
        input.into_iter().partition(is_in_existing);

    SplitIdsResult {
        in_input_in_invoice,
        in_input_not_in_invoice,
        not_in_input_in_invoice,
    }
}
