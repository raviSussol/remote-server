use crate::{
    business::{
        InsertSupplierInvoiceError, InsertSupplierInvoiceLineError, InsertSupplierInvoiceLineErrors,
    },
    database::repository::{ItemRepository, RepositoryError},
    server::service::graphql::schema::mutations::supplier_invoice::InsertSupplierInvoiceLineInput,
};

pub async fn check_item_id_insert(
    lines: &Vec<InsertSupplierInvoiceLineInput>,
    repository: &ItemRepository,
) -> Result<Vec<InsertSupplierInvoiceLineErrors>, InsertSupplierInvoiceError> {
    let lines_with_item_id: Vec<LineWithItemId> = lines.iter().map(LineWithItemId::from).collect();

    let error_lines = get_lines_with_wrong_item_ids(lines_with_item_id, repository).await?;

    Ok(error_lines
        .into_iter()
        .map(InsertSupplierInvoiceLineErrors::from)
        .collect())
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
