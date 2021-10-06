use chrono::NaiveDateTime;

use super::{DatetimeFilter, EqualFilter, SimpleStringFilter, Sort};

#[derive(PartialEq, Debug, Clone)]
pub enum InvoiceStatus {
    Draft,
    Confirmed,
    Finalised,
}
#[derive(PartialEq, Debug, Clone)]
pub enum InvoiceType {
    CustomerInvoice,
    SupplierInvoice,
}

#[derive(PartialEq, Debug)]
pub struct Invoice {
    pub id: String,
    pub other_party_name: String,
    pub other_party_id: String,
    pub status: InvoiceStatus,
    pub r#type: InvoiceType,
    pub invoice_number: i32,
    pub their_reference: Option<String>,
    pub comment: Option<String>,
    pub entry_datetime: NaiveDateTime,
    pub confirm_datetime: Option<NaiveDateTime>,
    pub finalised_datetime: Option<NaiveDateTime>,
}

pub struct InvoiceFilter {
    pub name_id: Option<EqualFilter<String>>,
    pub store_id: Option<EqualFilter<String>>,
    pub r#type: Option<EqualFilter<InvoiceType>>,
    pub status: Option<EqualFilter<InvoiceStatus>>,
    pub comment: Option<SimpleStringFilter>,
    pub their_reference: Option<EqualFilter<String>>,
    pub entry_datetime: Option<DatetimeFilter>,
    pub confirm_datetime: Option<DatetimeFilter>,
    pub finalised_datetime: Option<DatetimeFilter>,
}

pub enum InvoiceSortField {
    Type,
    Status,
    EntryDatetime,
    ConfirmDatetime,
    FinalisedDateTime,
}

pub type InvoiceSort = Sort<InvoiceSortField>;
