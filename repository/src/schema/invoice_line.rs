use super::diesel_schema::invoice_line;

use chrono::NaiveDate;
use diesel_derive_enum::DbEnum;

#[derive(DbEnum, Debug, Clone, PartialEq, Eq)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum InvoiceLineRowType {
    StockIn,
    StockOut,
    UnallocatedStock,
    Service,
}

impl Default for InvoiceLineRowType {
    fn default() -> Self {
        Self::StockIn
    }
}

#[derive(Clone, Queryable, Insertable, AsChangeset, Debug, PartialEq, Default)]
#[table_name = "invoice_line"]
pub struct InvoiceLineRow {
    pub id: String,
    pub invoice_id: String,
    pub item_id: String,
    pub item_name: String,
    pub item_code: String,
    pub stock_line_id: Option<String>,
    pub location_id: Option<String>,
    pub batch: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub pack_size: i32,
    pub cost_price_per_pack: f64,
    /// Sell price before tax
    pub sell_price_per_pack: f64,
    pub total_before_tax: f64,
    pub total_after_tax: f64,
    /// Optional column to store line a line specific tax value
    pub tax: Option<f64>,
    #[column_name = "type_"]
    pub r#type: InvoiceLineRowType,
    pub number_of_packs: i32,
    pub note: Option<String>,
}
