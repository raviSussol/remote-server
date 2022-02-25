use chrono::NaiveDate;

use crate::schema::{StocktakeRow, StocktakeStatus};

pub fn mock_stocktake_a() -> StocktakeRow {
    StocktakeRow {
        id: "stocktake_a".to_string(),
        store_id: "store_a".to_string(),
        stocktake_number: 9,
        comment: None,
        description: None,
        status: StocktakeStatus::New,
        created_datetime: NaiveDate::from_ymd(2021, 12, 14).and_hms_milli(12, 30, 0, 0),
        finalised_datetime: None,
        inventory_adjustment_id: None,
    }
}

pub fn mock_stocktake_b() -> StocktakeRow {
    StocktakeRow {
        id: "stocktake_b".to_string(),
        store_id: "store_b".to_string(),
        stocktake_number: 10,
        comment: Some("stocktake_comment_b".to_string()),
        description: Some("stocktake_description_b".to_string()),
        status: StocktakeStatus::New,
        created_datetime: NaiveDate::from_ymd(2021, 12, 14).and_hms_milli(22, 15, 0, 0),
        finalised_datetime: None,
        inventory_adjustment_id: None,
    }
}

pub fn mock_stocktake_no_line_a() -> StocktakeRow {
    StocktakeRow {
        id: "no_line_a".to_string(),
        store_id: "store_a".to_string(),
        stocktake_number: 10,
        comment: Some("stocktake_no_line_comment_a".to_string()),
        description: Some("stocktake_no_line_description_a".to_string()),
        status: StocktakeStatus::New,
        created_datetime: NaiveDate::from_ymd(2022, 2, 9).and_hms_milli(11, 15, 0, 0),
        finalised_datetime: None,
        inventory_adjustment_id: None,
    }
}

pub fn mock_stocktake_no_line_b() -> StocktakeRow {
    StocktakeRow {
        id: "no_line_b".to_string(),
        store_id: "store_a".to_string(),
        stocktake_number: 10,
        comment: Some("stocktake_no_line_comment_b".to_string()),
        description: Some("stocktake_no_line_description_b".to_string()),
        status: StocktakeStatus::New,
        created_datetime: NaiveDate::from_ymd(2022, 2, 10).and_hms_milli(12, 15, 0, 0),
        finalised_datetime: None,
        inventory_adjustment_id: None,
    }
}

pub fn mock_stocktake_data() -> Vec<StocktakeRow> {
    vec![
        mock_stocktake_a(),
        mock_stocktake_b(),
        mock_stocktake_no_line_a(),
        mock_stocktake_no_line_b(),
    ]
}