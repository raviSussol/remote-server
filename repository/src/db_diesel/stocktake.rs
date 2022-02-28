use crate::{DatetimeFilter, EqualFilter, Pagination, Sort};

use crate::{
    diesel_macros::{apply_date_time_filter, apply_equal_filter, apply_sort},
    schema::{
        diesel_schema::stocktake::{self, dsl as stocktake_dsl},
        StocktakeRow, StocktakeStatus,
    },
    DBType, RepositoryError, StorageConnection,
};

use diesel::{dsl::IntoBoxed, prelude::*};

#[derive(Clone)]
pub struct StocktakeFilter {
    pub id: Option<EqualFilter<String>>,
    pub store_id: Option<EqualFilter<String>>,
    pub stocktake_number: Option<EqualFilter<i64>>,
    pub status: Option<EqualFilter<StocktakeStatus>>,
    pub created_datetime: Option<DatetimeFilter>,
    pub finalised_datetime: Option<DatetimeFilter>,
}

impl StocktakeFilter {
    pub fn new() -> StocktakeFilter {
        StocktakeFilter {
            id: None,
            store_id: None,
            stocktake_number: None,
            status: None,
            created_datetime: None,
            finalised_datetime: None,
        }
    }

    pub fn id(mut self, filter: EqualFilter<String>) -> Self {
        self.id = Some(filter);
        self
    }

    pub fn store_id(mut self, filter: EqualFilter<String>) -> Self {
        self.store_id = Some(filter);
        self
    }

    pub fn stocktake_number(mut self, filter: EqualFilter<i64>) -> Self {
        self.stocktake_number = Some(filter);
        self
    }

    pub fn status(mut self, filter: EqualFilter<StocktakeStatus>) -> Self {
        self.status = Some(filter);
        self
    }

    pub fn created_datetime(mut self, filter: DatetimeFilter) -> Self {
        self.created_datetime = Some(filter);
        self
    }

    pub fn finalised_datetime(mut self, filter: DatetimeFilter) -> Self {
        self.finalised_datetime = Some(filter);
        self
    }
}

pub enum StocktakeSortField {
    Status,
    CreatedDatetime,
    FinalisedDatetime,
}

pub type Stocktake = StocktakeRow;

pub type StocktakeSort = Sort<StocktakeSortField>;

type BoxedStocktakeQuery = IntoBoxed<'static, stocktake::table, DBType>;

pub fn create_filtered_query<'a>(filter: Option<StocktakeFilter>) -> BoxedStocktakeQuery {
    let mut query = stocktake_dsl::stocktake.into_boxed();

    if let Some(f) = filter {
        apply_equal_filter!(query, f.id, stocktake::id);
        apply_equal_filter!(query, f.store_id, stocktake::store_id);
        apply_equal_filter!(query, f.stocktake_number, stocktake::stocktake_number);

        if let Some(value) = f.status {
            if let Some(eq) = value.equal_to {
                query = query.filter(stocktake::status.eq(eq));
            }
        }
        apply_date_time_filter!(query, f.created_datetime, stocktake::created_datetime);
        apply_date_time_filter!(query, f.finalised_datetime, stocktake::finalised_datetime);
    }
    query
}

pub struct StocktakeRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> StocktakeRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        StocktakeRepository { connection }
    }

    pub fn count(&self, filter: Option<StocktakeFilter>) -> Result<i64, RepositoryError> {
        // TODO (beyond M1), check that store_id matches current store
        let query = create_filtered_query(filter);

        Ok(query.count().get_result(&self.connection.connection)?)
    }

    pub fn query_by_filter(
        &self,
        filter: StocktakeFilter,
    ) -> Result<Vec<Stocktake>, RepositoryError> {
        self.query(Pagination::new(), Some(filter), None)
    }

    /// Gets all invoices
    pub fn query(
        &self,
        pagination: Pagination,
        filter: Option<StocktakeFilter>,
        sort: Option<StocktakeSort>,
    ) -> Result<Vec<Stocktake>, RepositoryError> {
        let mut query = create_filtered_query(filter);

        if let Some(sort) = sort {
            match sort.key {
                StocktakeSortField::Status => apply_sort!(query, sort, stocktake_dsl::status),
                StocktakeSortField::CreatedDatetime => {
                    apply_sort!(query, sort, stocktake_dsl::created_datetime)
                }
                StocktakeSortField::FinalisedDatetime => {
                    apply_sort!(query, sort, stocktake_dsl::finalised_datetime)
                }
            }
        } else {
            query = query.order(stocktake_dsl::id.asc())
        }

        let result = query
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64)
            .load::<Stocktake>(&self.connection.connection)?;

        Ok(result)
    }

    pub fn find_one_by_id(&self, row_id: &str) -> Result<Option<Stocktake>, RepositoryError> {
        Ok(stocktake_dsl::stocktake
            .filter(stocktake_dsl::id.eq(row_id))
            .first::<Stocktake>(&self.connection.connection)
            .optional()?)
    }
}
