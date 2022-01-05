use domain::EqualFilter;
use repository::{
    schema::StockTakeLineRow, RepositoryError, StockTakeLine, StockTakeLineFilter,
    StockTakeLineRepository, StockTakeLineRowRepository, StorageConnection,
};

use crate::{
    service_provider::ServiceContext,
    stock_take::validate::{check_stock_take_exist, check_stock_take_not_finalized},
    validate::check_store_id_matches,
};

use super::{
    query::get_stock_take_line,
    validate::{check_location_exists, check_stock_line_exist},
};

pub struct InsertStockTakeLineInput {
    pub id: String,
    pub stock_take_id: String,
    pub stock_line_id: String,
    pub location_id: Option<String>,
    pub batch: Option<String>,
    pub comment: Option<String>,
    pub cost_price_pack: f64,
    pub sell_price_pack: f64,
    pub snapshot_number_of_packs: i32,
    pub counted_number_of_packs: i32,
}

#[derive(Debug, PartialEq)]
pub enum InsertStockTakeLineError {
    DatabaseError(RepositoryError),
    InternalError(String),
    InvalidStore,
    StockTakeDoesNotExist,
    StockTakeLineAlreadyExists,
    StockLineDoesNotExist,
    LocationDoesNotExist,
    CannotEditFinalised,
}

fn check_stock_take_line_does_not_exist(
    connection: &StorageConnection,
    id: &str,
) -> Result<bool, RepositoryError> {
    let count = StockTakeLineRepository::new(connection).count(Some(
        StockTakeLineFilter::new().id(EqualFilter::equal_to(id)),
    ))?;
    Ok(count == 0)
}

fn validate(
    connection: &StorageConnection,
    store_id: &str,
    stock_take_line: &InsertStockTakeLineInput,
) -> Result<(), InsertStockTakeLineError> {
    let stock_take = match check_stock_take_exist(connection, &stock_take_line.stock_take_id)? {
        Some(stock_take) => stock_take,
        None => return Err(InsertStockTakeLineError::StockTakeDoesNotExist),
    };
    if !check_stock_take_not_finalized(&stock_take.status) {
        return Err(InsertStockTakeLineError::CannotEditFinalised);
    }
    if !check_store_id_matches(store_id, &stock_take.store_id) {
        return Err(InsertStockTakeLineError::InvalidStore);
    }
    if !check_stock_take_line_does_not_exist(connection, &stock_take_line.id)? {
        return Err(InsertStockTakeLineError::StockTakeLineAlreadyExists);
    }
    if !check_stock_line_exist(connection, &stock_take_line.stock_line_id)? {
        return Err(InsertStockTakeLineError::StockLineDoesNotExist);
    }

    if let Some(location_id) = &stock_take_line.location_id {
        if !check_location_exists(connection, location_id)? {
            return Err(InsertStockTakeLineError::LocationDoesNotExist);
        }
    }

    Ok(())
}

fn generate(
    InsertStockTakeLineInput {
        id,
        stock_take_id,
        stock_line_id,
        location_id,
        batch,
        comment,
        cost_price_pack,
        sell_price_pack,
        snapshot_number_of_packs,
        counted_number_of_packs,
    }: InsertStockTakeLineInput,
) -> StockTakeLineRow {
    StockTakeLineRow {
        id,
        stock_take_id,
        stock_line_id,
        location_id,
        batch,
        comment,
        cost_price_pack,
        sell_price_pack,
        snapshot_number_of_packs,
        counted_number_of_packs,
    }
}

pub fn insert_stock_take_line(
    ctx: &ServiceContext,
    store_id: &str,
    input: InsertStockTakeLineInput,
) -> Result<StockTakeLine, InsertStockTakeLineError> {
    let result = ctx
        .connection
        .transaction_sync(|connection| {
            validate(connection, store_id, &input)?;
            let new_stock_take_line = generate(input);
            StockTakeLineRowRepository::new(&connection).upsert_one(&new_stock_take_line)?;

            let line = get_stock_take_line(ctx, new_stock_take_line.id)?;
            line.ok_or(InsertStockTakeLineError::InternalError(
                "Failed to read the just inserted stock take line!".to_string(),
            ))
        })
        .map_err(|error| error.to_inner_error())?;
    Ok(result)
}

impl From<RepositoryError> for InsertStockTakeLineError {
    fn from(error: RepositoryError) -> Self {
        InsertStockTakeLineError::DatabaseError(error)
    }
}