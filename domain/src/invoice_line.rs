use chrono::NaiveDate;

use super::{EqualFilter, Sort};
#[derive(Clone)]

pub struct InvoiceLine {
    pub id: String,
    pub stock_line_id: Option<String>,
    pub invoice_id: String,
    pub location_id: Option<String>,
    pub location_name: Option<String>,
    pub item_id: String,
    pub item_name: String,
    pub item_code: String,
    pub pack_size: i32,
    pub number_of_packs: i32,
    pub cost_price_per_pack: f64,
    pub sell_price_per_pack: f64,
    pub batch: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub note: Option<String>,
}

pub struct InvoiceLineFilter {
    pub id: Option<EqualFilter<String>>,
    pub invoice_id: Option<EqualFilter<String>>,
}

impl InvoiceLineFilter {
    pub fn new() -> InvoiceLineFilter {
        InvoiceLineFilter {
            id: None,
            invoice_id: None,
        }
    }

    pub fn match_id(mut self, id: &str) -> Self {
        self.id = Some(EqualFilter {
            equal_to: Some(id.to_owned()),
            equal_any: None,
        });

        self
    }

    pub fn match_ids(mut self, ids: Vec<String>) -> Self {
        self.id = Some(EqualFilter {
            equal_to: None,
            equal_any: Some(ids),
        });

        self
    }

    pub fn match_invoice_id(mut self, id: &str) -> Self {
        self.invoice_id = Some(EqualFilter {
            equal_to: Some(id.to_owned()),
            equal_any: None,
        });

        self
    }

    pub fn match_invoice_ids(mut self, ids: Vec<String>) -> Self {
        self.invoice_id = Some(EqualFilter {
            equal_to: None,
            equal_any: Some(ids),
        });

        self
    }
}

pub type InvoiceLineSort = Sort<()>;
