pub mod invoice;
pub mod item;
pub mod name;
use chrono::NaiveDateTime;

pub struct SimpleStringFilter {
    pub equal_to: Option<String>,
    pub like: Option<String>,
}

pub struct EqualFilter<T> {
    pub equal_to: Option<T>,
}

pub struct DatetimeFilter {
    pub equal_to: Option<NaiveDateTime>,
    pub before_or_equal_to: Option<NaiveDateTime>,
    pub after_or_equal_to: Option<NaiveDateTime>,
}

pub struct Sort<T> {
    pub key: T,
    pub desc: Option<bool>,
}

pub const DEFAULT_LIMIT: u32 = 100;

pub struct PaginationOption {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
}
