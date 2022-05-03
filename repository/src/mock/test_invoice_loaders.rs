use chrono::NaiveDate;
use util::inline_init;

use crate::{
    requisition_row::{RequisitionRowStatus, RequisitionRowType},
    InvoiceRow, InvoiceRowStatus, InvoiceRowType, RequisitionRow,
};

use super::{mock_name_store_b, mock_store_a, MockData};

pub fn mock_test_invoice_loaders() -> MockData {
    let mut result = MockData::default();
    result.invoices.push(mock_invoice_loader_invoice1());
    result.invoices.push(mock_invoice_loader_invoice2());
    result.requisitions.push(mock_invoice_loader_requistion1());
    result
}

pub fn mock_invoice_loader_requistion1() -> RequisitionRow {
    inline_init(|r: &mut RequisitionRow| {
        r.id = "mock_invoice_loader_requistion1".to_owned();
        r.requisition_number = 1;
        r.name_id = "name_a".to_owned();
        r.store_id = mock_store_a().id;
        r.r#type = RequisitionRowType::Request;
        r.status = RequisitionRowStatus::Draft;
        r.created_datetime = NaiveDate::from_ymd(2021, 01, 01).and_hms(0, 0, 0);
        r.max_months_of_stock = 1.0;
        r.min_months_of_stock = 0.9;
    })
}

pub fn mock_invoice_loader_invoice1() -> InvoiceRow {
    inline_init(|r: &mut InvoiceRow| {
        r.id = "mock_invoice_loader_invoice1".to_string();
        r.name_id = mock_name_store_b().id;
        r.store_id = mock_store_a().id;
        r.invoice_number = 1;
        r.requisition_id = Some(mock_invoice_loader_requistion1().id);
        r.r#type = InvoiceRowType::OutboundShipment;
        r.status = InvoiceRowStatus::Picked;
        r.created_datetime = NaiveDate::from_ymd(1970, 1, 1).and_hms_milli(12, 30, 0, 0);
    })
}

pub fn mock_invoice_loader_invoice2() -> InvoiceRow {
    inline_init(|r: &mut InvoiceRow| {
        r.id = "mock_invoice_loader_invoice2".to_string();
        r.name_id = mock_name_store_b().id;
        r.store_id = mock_store_a().id;
        r.invoice_number = 1;
        r.r#type = InvoiceRowType::OutboundShipment;
        r.status = InvoiceRowStatus::Picked;
        r.created_datetime = NaiveDate::from_ymd(1970, 1, 1).and_hms_milli(12, 30, 0, 0);
        r.linked_invoice_id = Some(mock_invoice_loader_invoice1().id);
    })
}
