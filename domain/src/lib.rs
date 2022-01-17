pub mod document;
pub mod inbound_shipment;
pub mod invoice;
pub mod invoice_line;
pub mod item;
pub mod json_schema;
pub mod location;
pub mod master_list;
pub mod master_list_line;
pub mod name;
pub mod outbound_shipment;
pub mod shipment_tax_update;
pub mod stock_line;

use chrono::{NaiveDate, NaiveDateTime};

#[derive(Clone, PartialEq, Debug)]
pub struct SimpleStringFilter {
    pub equal_to: Option<String>,
    pub like: Option<String>,
}

impl SimpleStringFilter {
    pub fn equal_to(value: &str) -> Self {
        SimpleStringFilter {
            equal_to: Some(value.to_owned()),
            like: None,
        }
    }

    pub fn like(value: &str) -> Self {
        SimpleStringFilter {
            equal_to: None,
            like: Some(value.to_owned()),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct EqualFilter<T> {
    pub equal_to: Option<T>,
    pub not_equal_to: Option<T>,
    pub equal_any: Option<Vec<T>>,
}

impl EqualFilter<String> {
    pub fn equal_to(value: &str) -> Self {
        EqualFilter {
            equal_to: Some(value.to_owned()),
            not_equal_to: None,
            equal_any: None,
        }
    }

    pub fn not_equal_to(value: &str) -> Self {
        EqualFilter {
            equal_to: None,
            not_equal_to: Some(value.to_owned()),
            equal_any: None,
        }
    }

    pub fn equal_any(value: Vec<String>) -> Self {
        EqualFilter {
            equal_to: None,
            not_equal_to: None,
            equal_any: Some(value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DatetimeFilter {
    pub equal_to: Option<NaiveDateTime>,
    pub before_or_equal_to: Option<NaiveDateTime>,
    pub after_or_equal_to: Option<NaiveDateTime>,
}

impl DatetimeFilter {
    pub fn date_range(from: NaiveDateTime, to: NaiveDateTime) -> DatetimeFilter {
        DatetimeFilter {
            equal_to: None,
            after_or_equal_to: Some(from),
            before_or_equal_to: Some(to),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DateFilter {
    pub equal_to: Option<NaiveDate>,
    pub before_or_equal_to: Option<NaiveDate>,
    pub after_or_equal_to: Option<NaiveDate>,
}

impl DateFilter {
    pub fn date_range(from: NaiveDate, to: NaiveDate) -> DateFilter {
        DateFilter {
            equal_to: None,
            after_or_equal_to: Some(from),
            before_or_equal_to: Some(to),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Sort<T> {
    pub key: T,
    pub desc: Option<bool>,
}

pub const DEFAULT_LIMIT: u32 = 100;

#[derive(Debug, PartialEq)]
pub struct PaginationOption {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
}

impl Pagination {
    pub fn new() -> Pagination {
        Pagination {
            offset: 0,
            limit: DEFAULT_LIMIT,
        }
    }

    pub fn all() -> Pagination {
        Pagination {
            offset: 0,
            limit: std::u32::MAX,
        }
    }

    pub fn one() -> Pagination {
        Pagination {
            offset: 0,
            limit: 1,
        }
    }
}
