use super::{
    location_row::location, stock_line_row::stock_line,
    stocktake_line_row::stocktake_line::dsl as stocktake_line_dsl, stocktake_row::stocktake,
    StorageConnection,
};

use crate::repository_error::RepositoryError;

use diesel::prelude::*;

use chrono::NaiveDate;

table! {
    stocktake_line (id) {
        id -> Text,
        stocktake_id -> Text,
        stock_line_id -> Nullable<Text>,
        location_id	-> Nullable<Text>,
        comment	-> Nullable<Text>,
        snapshot_number_of_packs -> Integer,
        counted_number_of_packs -> Nullable<Integer>,

        // stock line related fields:
        item_id -> Text,
        batch -> Nullable<Text>,
        expiry_date -> Nullable<Date>,
        pack_size -> Nullable<Integer>,
        cost_price_per_pack -> Nullable<Double>,
        sell_price_per_pack -> Nullable<Double>,
        note -> Nullable<Text>,
    }
}

joinable!(stocktake_line -> location (location_id));
joinable!(stocktake_line -> stocktake (stocktake_id));
joinable!(stocktake_line -> stock_line (stock_line_id));

#[derive(Clone, Queryable, Insertable, AsChangeset, Debug, PartialEq, Default)]
#[table_name = "stocktake_line"]
pub struct StocktakeLineRow {
    pub id: String,
    pub stocktake_id: String,
    /// If missing, a new stock line needs to be created when finalizing the stocktake
    pub stock_line_id: Option<String>,
    pub location_id: Option<String>,
    /// Comment for this stocktake line
    pub comment: Option<String>,
    pub snapshot_number_of_packs: i32,
    pub counted_number_of_packs: Option<i32>,

    // stock line related fields:
    /// When a creating a new stock line this field holds the required item id
    pub item_id: String,
    pub batch: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub pack_size: Option<i32>,
    pub cost_price_per_pack: Option<f64>,
    pub sell_price_per_pack: Option<f64>,
    pub note: Option<String>,
}

pub struct StocktakeLineRowRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> StocktakeLineRowRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        StocktakeLineRowRepository { connection }
    }

    #[cfg(feature = "postgres")]
    pub fn upsert_one(&self, row: &StocktakeLineRow) -> Result<(), RepositoryError> {
        diesel::insert_into(stocktake_line_dsl::stocktake_line)
            .values(row)
            .on_conflict(stocktake_line_dsl::id)
            .do_update()
            .set(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    #[cfg(not(feature = "postgres"))]
    pub fn upsert_one(&self, row: &StocktakeLineRow) -> Result<(), RepositoryError> {
        diesel::replace_into(stocktake_line_dsl::stocktake_line)
            .values(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        diesel::delete(stocktake_line_dsl::stocktake_line.filter(stocktake_line_dsl::id.eq(id)))
            .execute(&self.connection.connection)?;
        Ok(())
    }

    pub fn find_one_by_id(&self, id: &str) -> Result<Option<StocktakeLineRow>, RepositoryError> {
        let result = stocktake_line_dsl::stocktake_line
            .filter(stocktake_line_dsl::id.eq(id))
            .first(&self.connection.connection)
            .optional();
        result.map_err(|err| RepositoryError::from(err))
    }

    pub fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<StocktakeLineRow>, RepositoryError> {
        let result = stocktake_line_dsl::stocktake_line
            .filter(stocktake_line_dsl::id.eq_any(ids))
            .load(&self.connection.connection)?;
        Ok(result)
    }
}
