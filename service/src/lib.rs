use repository::RepositoryError;
use repository::{Pagination, PaginationOption, DEFAULT_PAGINATION_LIMIT};
use std::convert::TryInto;

pub mod auth_data;
pub mod dashboard;
pub mod invoice;
pub mod invoice_line;
pub mod item;
pub mod item_stats;
pub mod location;
pub mod master_list;
pub mod name;
pub mod number;
pub mod permission_validation;
pub mod permissions;
pub mod requisition;
pub mod requisition_line;
pub mod service_provider;
pub mod stock_line;
pub mod stocktake;
pub mod stocktake_line;
pub mod store;
pub mod sync_processor;
pub mod token;
pub mod token_bucket;
pub mod user_account;
pub mod validate;

#[derive(PartialEq, Debug)]
pub struct ListResult<T> {
    pub rows: Vec<T>,
    pub count: u32,
}

impl<T> ListResult<T> {
    pub fn empty() -> ListResult<T> {
        ListResult {
            rows: vec![],
            count: 0,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ListError {
    DatabaseError(RepositoryError),
    LimitBelowMin(u32),
    LimitAboveMax(u32),
}
#[derive(PartialEq, Debug)]
pub enum SingleRecordError {
    DatabaseError(RepositoryError),
    NotFound(String),
}

pub enum WithDBError<T> {
    DatabaseError(RepositoryError),
    Error(T),
}

impl<T> WithDBError<T> {
    pub fn db(error: RepositoryError) -> Self {
        WithDBError::DatabaseError(error)
    }

    pub fn err(error: T) -> Self {
        WithDBError::Error(error)
    }
}

impl<T> From<RepositoryError> for WithDBError<T> {
    fn from(error: RepositoryError) -> Self {
        WithDBError::DatabaseError(error)
    }
}

impl From<RepositoryError> for ListError {
    fn from(error: RepositoryError) -> Self {
        ListError::DatabaseError(error)
    }
}

impl From<RepositoryError> for SingleRecordError {
    fn from(error: RepositoryError) -> Self {
        SingleRecordError::DatabaseError(error)
    }
}

pub fn get_default_pagination(
    pagination_option: Option<PaginationOption>,
    max_limit: u32,
    min_limit: u32,
) -> Result<Pagination, ListError> {
    let check_limit = |limit: u32| -> Result<u32, ListError> {
        if limit < min_limit {
            return Err(ListError::LimitBelowMin(min_limit));
        }
        if limit > max_limit {
            return Err(ListError::LimitAboveMax(max_limit));
        }

        Ok(limit)
    };

    let result = if let Some(pagination) = pagination_option {
        Pagination {
            offset: pagination.offset.unwrap_or(0),
            limit: match pagination.limit {
                Some(limit) => check_limit(limit)?,
                None => DEFAULT_PAGINATION_LIMIT,
            },
        }
    } else {
        Pagination {
            offset: 0,
            limit: DEFAULT_PAGINATION_LIMIT,
        }
    };

    Ok(result)
}

// TODO move the following methods to util

pub fn i32_to_u32(num: i32) -> u32 {
    num.try_into().unwrap_or(0)
}

pub fn i64_to_u32(num: i64) -> u32 {
    num.try_into().unwrap_or(0)
}

pub fn usize_to_u32(num: usize) -> u32 {
    num.try_into().unwrap_or(0)
}

pub fn u32_to_i32(num: u32) -> i32 {
    num.try_into().unwrap_or(0)
}

#[derive(Debug, PartialEq)]
pub struct InputWithResult<I, R> {
    pub input: I,
    pub result: R,
}
