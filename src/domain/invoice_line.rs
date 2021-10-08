use chrono::NaiveDate;
#[derive(Clone)]

pub struct InvoiceLine {
    pub id: String,
    pub invoice_id: String,
    pub item_id: String,
    pub item_name: String,
    pub item_code: String,
    pub pack_size: i32,
    pub number_of_packs: i32,
    pub cost_price_per_pack: f64,
    pub sell_price_per_pack: f64,
    pub batch: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub stock_line: StockLine,
}

#[derive(Clone)]
pub struct StockLine {
    pub available_number_of_packs: i32,
}