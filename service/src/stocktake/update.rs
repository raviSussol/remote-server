use chrono::Utc;
use domain::{name::NameFilter, EqualFilter, SimpleStringFilter};
use repository::{
    schema::{
        InvoiceLineRow, InvoiceLineRowType, InvoiceRow, InvoiceRowStatus, InvoiceRowType,
        NumberRowType, StockLineRow, StocktakeRow, StocktakeStatus,
    },
    InvoiceLineRowRepository, InvoiceRepository, ItemRepository, NameQueryRepository,
    RepositoryError, StockLineRowRepository, Stocktake, StocktakeLine, StocktakeLineFilter,
    StocktakeLineRepository, StocktakeRowRepository, StorageConnection,
};
use util::{constants::INVENTORY_ADJUSTMENT_NAME_CODE, uuid::uuid};

use crate::{
    number::next_number, service_provider::ServiceContext, stocktake::query::get_stocktake,
    validate::check_store_id_matches,
};

use super::validate::{check_stocktake_exist, check_stocktake_not_finalised};

pub struct UpdateStocktakeInput {
    pub id: String,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub status: Option<StocktakeStatus>,
}

#[derive(Debug, PartialEq)]
pub enum UpdateStocktakeError {
    DatabaseError(RepositoryError),
    InternalError(String),
    InvalidStore,
    StocktakeDoesNotExist,
    CannotEditFinalised,
    /// Stocktakes doesn't contain any lines
    NoLines,
    /// Holds list of affected stock lines
    SnapshotCountCurrentCountMismatch(Vec<StocktakeLine>),
}

fn check_snapshot_matches_current_count(
    stocktake_lines: &[StocktakeLine],
) -> Option<Vec<StocktakeLine>> {
    let mut mismatches = Vec::new();
    for line in stocktake_lines {
        let stock_line = match &line.stock_line {
            Some(stock_line) => stock_line,
            None => continue,
        };
        if line.line.snapshot_number_of_packs != stock_line.total_number_of_packs {
            mismatches.push(line.clone());
        }
    }
    if !mismatches.is_empty() {
        return Some(mismatches);
    }
    None
}

fn load_stocktake_lines(
    connection: &StorageConnection,
    stocktake_id: &str,
) -> Result<Vec<StocktakeLine>, RepositoryError> {
    StocktakeLineRepository::new(connection).query_by_filter(
        StocktakeLineFilter::new().stocktake_id(EqualFilter::equal_to(stocktake_id)),
    )
}

fn validate(
    connection: &StorageConnection,
    store_id: &str,
    input: &UpdateStocktakeInput,
) -> Result<(StocktakeRow, Vec<StocktakeLine>), UpdateStocktakeError> {
    let existing = match check_stocktake_exist(connection, &input.id)? {
        Some(existing) => existing,
        None => return Err(UpdateStocktakeError::StocktakeDoesNotExist),
    };
    if !check_stocktake_not_finalised(&existing.status) {
        return Err(UpdateStocktakeError::CannotEditFinalised);
    }
    if !check_store_id_matches(store_id, &existing.store_id) {
        return Err(UpdateStocktakeError::InvalidStore);
    }
    let stocktake_lines = load_stocktake_lines(connection, &input.id)?;

    if let Some(StocktakeStatus::Finalised) = input.status {
        if stocktake_lines.len() == 0 {
            return Err(UpdateStocktakeError::NoLines);
        }

        if let Some(mismatches) = check_snapshot_matches_current_count(&stocktake_lines) {
            return Err(UpdateStocktakeError::SnapshotCountCurrentCountMismatch(
                mismatches,
            ));
        }
    }

    Ok((existing, stocktake_lines))
}

struct StocktakeGenerateJob {
    stocktake: StocktakeRow,

    // new inventory adjustment
    inventory_adjustment: Option<InvoiceRow>,
    inventory_adjustment_lines: Vec<InvoiceLineRow>,

    // list of stock_line upserts
    stock_lines: Vec<StockLineRow>,
}

/// Returns new stock line and matching invoice line
fn generate_stock_line_update(
    connection: &StorageConnection,
    invoice_id: &str,
    stocktake_line: &StocktakeLine,
    stock_line: &StockLineRow,
) -> Result<(StockLineRow, Option<InvoiceLineRow>), UpdateStocktakeError> {
    let counted_number_of_packs = stocktake_line
        .line
        .counted_number_of_packs
        .unwrap_or(stocktake_line.line.snapshot_number_of_packs);
    let delta = counted_number_of_packs - stocktake_line.line.snapshot_number_of_packs;
    let updated_line = StockLineRow {
        id: stock_line.id.clone(),
        item_id: stock_line.item_id.clone(),
        store_id: stock_line.store_id.clone(),
        location_id: stock_line.location_id.clone(),
        batch: stock_line.batch.clone(),
        pack_size: stock_line.pack_size,
        cost_price_per_pack: stock_line.cost_price_per_pack,
        sell_price_per_pack: stock_line.sell_price_per_pack,
        // TODO might get negative!
        available_number_of_packs: stock_line.available_number_of_packs + delta,
        total_number_of_packs: stock_line.total_number_of_packs + delta,
        expiry_date: stock_line.expiry_date,
        on_hold: stock_line.on_hold,
        note: stock_line.note.clone(),
    };

    let item = match ItemRepository::new(connection).find_one_by_id(&stock_line.item_id)? {
        Some(item) => item,
        None => {
            return Err(UpdateStocktakeError::InternalError(format!(
                "Can't find item {} for existing stocktake line {}!",
                &stock_line.item_id, stocktake_line.line.id
            )))
        }
    };

    let quantiy_change = i32::abs(delta);
    let shipment_line = if quantiy_change > 0 {
        let line_type = if delta > 0 {
            InvoiceLineRowType::StockIn
        } else {
            InvoiceLineRowType::StockOut
        };
        Some(InvoiceLineRow {
            id: uuid(),
            r#type: line_type,
            invoice_id: invoice_id.to_string(),
            item_id: stock_line.item_id.clone(),
            item_name: item.name,
            item_code: item.code,
            stock_line_id: Some(stock_line.id.clone()),
            location_id: stock_line.location_id.clone(),
            batch: stock_line.batch.clone(),
            expiry_date: stock_line.expiry_date,
            pack_size: stock_line.pack_size,
            cost_price_per_pack: stock_line.cost_price_per_pack,
            sell_price_per_pack: stock_line.sell_price_per_pack,
            total_before_tax: 0.0,
            total_after_tax: 0.0,
            tax: None,
            number_of_packs: quantiy_change,
            note: stock_line.note.clone(),
        })
    } else {
        None
    };
    Ok((updated_line, shipment_line))
}

/// Returns new stock line and matching invoice line
fn generate_new_stock_line(
    connection: &StorageConnection,
    store_id: &str,
    invoice_id: &str,
    stocktake_line: StocktakeLine,
) -> Result<(StockLineRow, Option<InvoiceLineRow>), UpdateStocktakeError> {
    let counted_number_of_packs = stocktake_line.line.counted_number_of_packs.unwrap_or(0);
    let row = stocktake_line.line;
    let pack_size = row.pack_size.unwrap_or(0);
    let cost_price_per_pack = row.cost_price_per_pack.unwrap_or(0.0);
    let sell_price_per_pack = row.sell_price_per_pack.unwrap_or(0.0);
    let item_id = row.item_id;

    let new_line = StockLineRow {
        id: uuid(),
        item_id: item_id.clone(),
        store_id: store_id.to_string(),
        location_id: row.location_id.clone(),
        batch: row.batch.clone(),
        pack_size,
        cost_price_per_pack,
        sell_price_per_pack,
        available_number_of_packs: counted_number_of_packs,
        total_number_of_packs: counted_number_of_packs,
        expiry_date: row.expiry_date,
        on_hold: false,
        note: row.note.clone(),
    };

    let item = match ItemRepository::new(connection).find_one_by_id(&item_id)? {
        Some(item) => item,
        None => {
            return Err(UpdateStocktakeError::InternalError(format!(
                "Can't find item {} for new stocktake line {}!",
                &item_id, row.id
            )))
        }
    };
    let shipment_line = if counted_number_of_packs > 0 {
        Some(InvoiceLineRow {
            id: uuid(),
            r#type: InvoiceLineRowType::StockIn,
            invoice_id: invoice_id.to_string(),
            item_id,
            item_name: item.name,
            item_code: item.code,
            stock_line_id: Some(new_line.id.clone()),
            location_id: row.location_id,
            batch: row.batch,
            expiry_date: row.expiry_date,
            pack_size,
            cost_price_per_pack,
            sell_price_per_pack,
            total_before_tax: 0.0,
            total_after_tax: 0.0,
            tax: None,
            number_of_packs: counted_number_of_packs,
            note: row.note,
        })
    } else {
        None
    };
    Ok((new_line, shipment_line))
}

fn generate(
    connection: &StorageConnection,
    UpdateStocktakeInput {
        id: _,
        comment: input_comment,
        description: input_description,
        status: input_status,
    }: UpdateStocktakeInput,
    existing: StocktakeRow,
    stocktake_lines: Vec<StocktakeLine>,
    store_id: &str,
) -> Result<StocktakeGenerateJob, UpdateStocktakeError> {
    if input_status != Some(StocktakeStatus::Finalised) {
        // just update the existing stocktake
        let stocktake = StocktakeRow {
            id: existing.id,
            store_id: existing.store_id,
            stocktake_number: existing.stocktake_number,
            comment: input_comment.or(existing.comment),
            description: input_description.or(existing.description),
            status: input_status.unwrap_or(existing.status),
            created_datetime: existing.created_datetime,
            finalised_datetime: None,
            inventory_adjustment_id: None,
        };
        return Ok(StocktakeGenerateJob {
            stocktake: stocktake,
            inventory_adjustment: None,
            inventory_adjustment_lines: vec![],
            stock_lines: vec![],
        });
    }

    // finalise the stocktake
    let mut inventory_adjustment_lines: Vec<InvoiceLineRow> = Vec::new();
    let mut stock_lines: Vec<StockLineRow> = Vec::new();
    let shipment_id = uuid();
    for stocktake_line in stocktake_lines {
        let (stock_line, shipment_line) = if let Some(ref stock_line) = stocktake_line.stock_line {
            // adjust existing stock line
            generate_stock_line_update(connection, &shipment_id, &stocktake_line, stock_line)?
        } else {
            // create new stock line
            generate_new_stock_line(connection, store_id, &shipment_id, stocktake_line)?
        };
        stock_lines.push(stock_line);
        if let Some(shipment_line) = shipment_line {
            inventory_adjustment_lines.push(shipment_line);
        }
    }

    // find inventory adjustment name:
    let name_result = NameQueryRepository::new(connection).query_by_filter(
        NameFilter::new().code(SimpleStringFilter::equal_to(INVENTORY_ADJUSTMENT_NAME_CODE)),
    )?;
    let invad_name = name_result
        .first()
        .ok_or(UpdateStocktakeError::InternalError(
            "Missing inventory adjustment name".to_string(),
        ))?;

    // create a shipment even if there are no shipment lines
    let now = Utc::now().naive_utc();
    let shipment = InvoiceRow {
        id: shipment_id,
        name_id: invad_name.id.to_owned(),
        store_id: store_id.to_string(),
        invoice_number: next_number(connection, &NumberRowType::InventoryAdjustment, store_id)?,
        name_store_id: None,
        r#type: InvoiceRowType::InventoryAdjustment,
        status: InvoiceRowStatus::Verified,
        on_hold: false,
        comment: None,
        their_reference: None,
        created_datetime: now.clone(),
        allocated_datetime: None,
        picked_datetime: None,
        shipped_datetime: None,
        delivered_datetime: None,
        verified_datetime: Some(now.clone()),
        colour: None,
        requisition_id: None,
        linked_invoice_id: None,
    };

    let stocktake = StocktakeRow {
        id: existing.id,
        store_id: existing.store_id,
        stocktake_number: existing.stocktake_number,
        comment: input_comment.or(existing.comment),
        description: input_description.or(existing.description),
        status: input_status.unwrap_or(existing.status),
        created_datetime: existing.created_datetime,
        finalised_datetime: Some(now),
        inventory_adjustment_id: Some(shipment.id.clone()),
    };

    Ok(StocktakeGenerateJob {
        stocktake,
        inventory_adjustment: Some(shipment),
        inventory_adjustment_lines,
        stock_lines,
    })
}

pub fn update_stocktake(
    ctx: &ServiceContext,
    store_id: &str,
    input: UpdateStocktakeInput,
) -> Result<Stocktake, UpdateStocktakeError> {
    let result = ctx
        .connection
        .transaction_sync(|connection| {
            let stocktake_id = input.id.clone();
            let (existing, stocktake_lines) = validate(connection, store_id, &input)?;
            let result = generate(connection, input, existing, stocktake_lines, store_id)?;

            // write data to the DB
            // write new stock lines
            let stock_line_repo = StockLineRowRepository::new(connection);
            for stock_line in result.stock_lines {
                stock_line_repo.upsert_one(&stock_line)?;
            }
            // write inventory adjustment
            if let Some(inventory_adjustment) = result.inventory_adjustment {
                let shipment_repo = InvoiceRepository::new(connection);
                shipment_repo.upsert_one(&inventory_adjustment)?;
            }
            let shipment_line_repo = InvoiceLineRowRepository::new(connection);
            for line in result.inventory_adjustment_lines {
                shipment_line_repo.upsert_one(&line)?;
            }
            StocktakeRowRepository::new(connection).upsert_one(&result.stocktake)?;

            // return the updated stocktake
            let stocktake = get_stocktake(ctx, stocktake_id)?;
            stocktake.ok_or(UpdateStocktakeError::InternalError(
                "Failed to read the just updated stocktake!".to_string(),
            ))
        })
        .map_err(|error| error.to_inner_error())?;
    Ok(result)
}

impl From<RepositoryError> for UpdateStocktakeError {
    fn from(error: RepositoryError) -> Self {
        UpdateStocktakeError::DatabaseError(error)
    }
}