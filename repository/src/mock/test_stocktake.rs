use chrono::NaiveDate;
use util::inline_init;

use crate::schema::{StockLineRow, StocktakeLineRow, StocktakeRow, StocktakeStatus};

use super::{mock_item_a, mock_stock_line_a, mock_stock_line_b, MockData};

pub fn mock_stocktake_without_lines() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "stocktake_without_lines".to_string();
        st.store_id = "store_a".to_string();
        st.stocktake_number = 1;
        st.created_datetime = NaiveDate::from_ymd(2021, 12, 14).and_hms_milli(12, 30, 0, 0);
    })
}

pub fn mock_stocktake_finalised() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "mock_stocktake_finalised".to_string();
        st.store_id = "store_a".to_string();
        st.stocktake_number = 2;
        st.status = StocktakeStatus::Finalised;
        st.created_datetime = NaiveDate::from_ymd(2021, 12, 14).and_hms_milli(12, 30, 0, 0);
        st.finalised_datetime =
            Some(NaiveDate::from_ymd(2021, 12, 20).and_hms_milli(10, 15, 10, 0));
    })
}

pub fn mock_stocktake_finalised_without_lines() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "mock_stocktake_finalised_no_lines".to_string();
        st.store_id = "store_a".to_string();
        st.stocktake_number = 3;
        st.status = StocktakeStatus::Finalised;
        st.created_datetime = NaiveDate::from_ymd(2021, 12, 15).and_hms_milli(12, 30, 0, 0);
        st.finalised_datetime =
            Some(NaiveDate::from_ymd(2021, 12, 21).and_hms_milli(10, 15, 10, 0));
    })
}

pub fn mock_stocktake_line_finalised() -> StocktakeLineRow {
    let stock_line = mock_stock_line_a();
    inline_init(|v: &mut StocktakeLineRow| {
        v.id = "stocktake_line_finalised".to_string();
        v.stocktake_id = mock_stocktake_finalised().id;
        v.stock_line_id = Some(stock_line.id);
        v.snapshot_number_of_packs = 11;
        v.counted_number_of_packs = Some(11);
        v.item_id = stock_line.item_id;
    })
}

// locked

pub fn mock_locked_stocktake() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "locked_stocktake".to_string();
        st.store_id = "store_a".to_string();
        st.status = StocktakeStatus::New;
        st.is_locked = true;
    })
}

pub fn mock_locked_stocktake_line() -> StocktakeLineRow {
    let stock_line = mock_stock_line_a();
    inline_init(|v: &mut StocktakeLineRow| {
        v.id = "locked stocktake_line_row".to_string();
        v.stocktake_id = mock_locked_stocktake().id;
        v.stock_line_id = Some(stock_line.id);
        v.item_id = stock_line.item_id;
    })
}

// stock surplus

pub fn mock_stocktake_stock_surplus() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "mock_stocktake_stock_surplus".to_string();
        st.store_id = "store_a".to_string();
        st.stocktake_number = 4;
        st.created_datetime = NaiveDate::from_ymd(2021, 12, 22).and_hms_milli(12, 31, 0, 0);
    })
}

pub fn mock_stock_line_stocktake_surplus() -> StockLineRow {
    StockLineRow {
        id: String::from("mock_stock_line_stocktake_surplus"),
        item_id: String::from("item_a"),
        location_id: None,
        store_id: String::from("store_a"),
        batch: Some(String::from("item_a_batch_b")),
        available_number_of_packs: 20,
        pack_size: 1,
        cost_price_per_pack: 0.0,
        sell_price_per_pack: 0.0,
        total_number_of_packs: 30,
        expiry_date: None,
        on_hold: false,
        note: None,
    }
}

pub fn mock_stocktake_line_stock_surplus() -> StocktakeLineRow {
    let stock_line = mock_stock_line_b();
    inline_init(|v: &mut StocktakeLineRow| {
        v.id = "mock_stocktake_line_stock_surplus".to_string();
        v.stocktake_id = mock_stocktake_stock_surplus().id;
        v.stock_line_id = Some(mock_stock_line_stocktake_surplus().id);
        v.snapshot_number_of_packs = stock_line.total_number_of_packs;
        v.counted_number_of_packs = Some(stock_line.total_number_of_packs + 10);
        v.item_id = stock_line.item_id;
    })
}

// stock deficit

pub fn mock_stocktake_stock_deficit() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "mock_stocktake_stock_deficit".to_string();
        st.store_id = "store_a".to_string();
        st.stocktake_number = 1;
        st.created_datetime = NaiveDate::from_ymd(2021, 12, 22).and_hms_milli(12, 31, 0, 0);
    })
}

pub fn mock_stock_line_stocktake_deficit() -> StockLineRow {
    StockLineRow {
        id: String::from("mock_stock_line_stocktake_deficit"),
        item_id: String::from("item_a"),
        location_id: None,
        store_id: String::from("store_a"),
        batch: Some(String::from("item_a_batch_b")),
        available_number_of_packs: 20,
        pack_size: 1,
        cost_price_per_pack: 0.0,
        sell_price_per_pack: 0.0,
        total_number_of_packs: 30,
        expiry_date: None,
        on_hold: false,
        note: None,
    }
}

pub fn mock_stocktake_line_stock_deficit() -> StocktakeLineRow {
    let stock_line = mock_stock_line_b();
    inline_init(|v: &mut StocktakeLineRow| {
        v.id = "mock_stocktake_line_stock_deficit".to_string();
        v.stocktake_id = mock_stocktake_stock_deficit().id;
        v.stock_line_id = Some(mock_stock_line_stocktake_deficit().id);
        v.snapshot_number_of_packs = stock_line.total_number_of_packs;
        v.counted_number_of_packs = Some(stock_line.total_number_of_packs - 10);
        v.item_id = mock_stock_line_stocktake_deficit().item_id;
    })
}

// stocktake without lines

pub fn mock_stocktake_no_lines() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "mock_stocktake_no_lines".to_string();
        st.store_id = "store_a".to_string();
        st.stocktake_number = 5;
        st.created_datetime = NaiveDate::from_ymd(2022, 1, 6).and_hms_milli(15, 31, 0, 0);
    })
}

// success: no count change should not generate shipment line

pub fn mock_stocktake_no_count_change() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "mock_stocktake_no_count_change".to_string();
        st.store_id = "store_a".to_string();
        st.stocktake_number = 8;
        st.created_datetime = NaiveDate::from_ymd(2022, 1, 6).and_hms_milli(16, 31, 0, 0);
    })
}

pub fn mock_stocktake_line_no_count_change() -> StocktakeLineRow {
    let stock_line = mock_stock_line_b();
    inline_init(|v: &mut StocktakeLineRow| {
        v.id = "mock_stocktake_line_no_count_change".to_string();
        v.stocktake_id = mock_stocktake_no_count_change().id;
        v.stock_line_id = Some(mock_stock_line_b().id);
        v.snapshot_number_of_packs = stock_line.total_number_of_packs;
        v.counted_number_of_packs = Some(stock_line.total_number_of_packs);
        v.item_id = stock_line.item_id;
    })
}

// stocktake full edit

pub fn mock_stocktake_full_edit() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "mock_stocktake_full_edit".to_string();
        st.store_id = "store_a".to_string();
        st.stocktake_number = 6;
        st.comment = Some("comment_0".to_string());
        st.description = Some("description_0".to_string());
        st.created_datetime = NaiveDate::from_ymd(2021, 12, 14).and_hms_milli(12, 32, 0, 0);
    })
}

// stocktake with new stock line

pub fn mock_stocktake_new_stock_line() -> StocktakeRow {
    inline_init(|st: &mut StocktakeRow| {
        st.id = "mock_stocktake_new_stock_line".to_string();
        st.store_id = "store_a".to_string();
        st.stocktake_number = 7;
        st.created_datetime = NaiveDate::from_ymd(2021, 12, 14).and_hms_milli(12, 33, 0, 0);
    })
}
pub fn mock_stocktake_line_new_stock_line() -> StocktakeLineRow {
    inline_init(|v: &mut StocktakeLineRow| {
        v.id = "mock_stocktake_line_new_stock_line".to_string();
        v.stocktake_id = mock_stocktake_new_stock_line().id;
        v.counted_number_of_packs = Some(55);
        v.item_id = mock_item_a().id;
        v.expiry_date = Some(NaiveDate::from_ymd(2022, 12, 14));
        v.batch = Some("batch".to_string());
        v.pack_size = Some(10);
        v.cost_price_per_pack = Some(11.0);
        v.sell_price_per_pack = Some(12.0);
        v.note = Some("note".to_string());
    })
}

pub fn test_stocktake_data() -> MockData {
    let mut data: MockData = Default::default();
    data.stocktakes = vec![
        mock_stocktake_without_lines(),
        mock_stocktake_finalised(),
        mock_stocktake_finalised_without_lines(),
        mock_stocktake_stock_surplus(),
        mock_stocktake_stock_deficit(),
        mock_stocktake_no_lines(),
        mock_stocktake_no_count_change(),
        mock_stocktake_full_edit(),
        mock_stocktake_new_stock_line(),
        mock_locked_stocktake(),
    ];
    data.stocktake_lines = vec![
        mock_stocktake_line_finalised(),
        mock_stocktake_line_stock_surplus(),
        mock_stocktake_line_stock_deficit(),
        mock_stocktake_line_no_count_change(),
        mock_stocktake_line_new_stock_line(),
        mock_locked_stocktake_line(),
    ];
    data.stock_lines = vec![
        mock_stock_line_stocktake_surplus(),
        mock_stock_line_stocktake_deficit(),
    ];
    data
}
