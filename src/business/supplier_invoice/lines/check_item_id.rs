use crate::{
    business::{
        InsertSupplierInvoiceLineError, InsertSupplierInvoiceLineErrors,
        UpsertSupplierInvoiceLineError,
    },
    database::repository::{ItemRepository, RepositoryError},
    server::service::graphql::schema::mutations::supplier_invoice::{
        InsertSupplierInvoiceLineInput, UpsertSupplierInvoiceLineInput,
    },
};

use super::UpsertSupplierInvoiceLineErrors;

pub async fn check_item_id_insert(
    lines: &Vec<InsertSupplierInvoiceLineInput>,
    repository: &ItemRepository,
) -> Result<Vec<InsertSupplierInvoiceLineErrors>, RepositoryError> {
    let lines_with_item_id: Vec<LineWithItemId> = lines.iter().map(LineWithItemId::from).collect();

    let error_lines = get_lines_with_wrong_item_ids(lines_with_item_id, repository).await?;

    Ok(error_lines
        .into_iter()
        .map(InsertSupplierInvoiceLineErrors::from)
        .collect())
}

pub async fn check_item_id_upsert(
    lines: &Vec<UpsertSupplierInvoiceLineInput>,
    repository: &ItemRepository,
) -> Result<Vec<UpsertSupplierInvoiceLineErrors>, RepositoryError> {
    let lines_with_item_id: Vec<LineWithItemId> =
        lines.iter().filter_map(OptLineWithItemId::from).collect();

    let error_lines = get_lines_with_wrong_item_ids(lines_with_item_id, repository).await?;

    Ok(error_lines
        .into_iter()
        .map(UpsertSupplierInvoiceLineErrors::from)
        .collect())
}

type OptLineWithItemId = Option<LineWithItemId>;

impl From<&UpsertSupplierInvoiceLineInput> for OptLineWithItemId {
    fn from(line: &UpsertSupplierInvoiceLineInput) -> Self {
        match &line.item_id {
            Some(item_id) => Some(LineWithItemId {
                id: line.id.clone(),
                item_id: item_id.clone(),
            }),
            None => None,
        }
    }
}

impl From<LineWithItemId> for UpsertSupplierInvoiceLineErrors {
    fn from(line: LineWithItemId) -> Self {
        UpsertSupplierInvoiceLineErrors {
            id: line.id.clone(),
            errors: vec![UpsertSupplierInvoiceLineError::ItemIdNotFound(
                line.item_id.clone(),
            )],
        }
    }
}

impl From<&InsertSupplierInvoiceLineInput> for LineWithItemId {
    fn from(line: &InsertSupplierInvoiceLineInput) -> Self {
        LineWithItemId {
            id: line.id.clone(),
            item_id: line.item_id.clone(),
        }
    }
}

impl From<LineWithItemId> for InsertSupplierInvoiceLineErrors {
    fn from(line: LineWithItemId) -> Self {
        InsertSupplierInvoiceLineErrors {
            id: line.id.clone(),
            errors: vec![InsertSupplierInvoiceLineError::ItemIdNotFound(
                line.item_id.clone(),
            )],
        }
    }
}

struct LineWithItemId {
    id: String,
    item_id: String,
}

async fn get_lines_with_wrong_item_ids(
    lines: Vec<LineWithItemId>,
    repository: &ItemRepository,
) -> Result<Vec<LineWithItemId>, RepositoryError> {
    let item_ids: Vec<String> = lines.iter().map(|input| input.item_id.clone()).collect();
    let items = repository.find_many_by_id(&item_ids).await?;

    let item_does_not_exists =
        |line: &LineWithItemId| -> bool { !items.iter().any(|item| item.id == line.item_id) };

    let result = lines.into_iter().filter(item_does_not_exists).collect();

    Ok(result)
}
