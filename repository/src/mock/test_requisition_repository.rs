use chrono::NaiveDate;
use util::inline_init;

use crate::schema::{RequisitionRow, RequisitionRowStatus, RequisitionRowType};

use super::MockData;

pub fn mock_test_requisition_repository() -> MockData {
    let mut result = MockData::default();
    result.requisitions.push(mock_request_draft_requisition());
    result.requisitions.push(mock_request_draft_requisition2());
    result
}

pub fn mock_request_draft_requisition() -> RequisitionRow {
    inline_init(|r: &mut RequisitionRow| {
        r.id = "mock_request_draft_requisition".to_owned();
        r.requisition_number = 1;
        r.name_id = "name_a".to_owned();
        r.store_id = "store_a".to_owned();
        r.r#type = RequisitionRowType::Request;
        r.status = RequisitionRowStatus::Draft;
        r.created_datetime = NaiveDate::from_ymd(2021, 01, 01).and_hms(0, 0, 0);
        r.max_months_of_stock = 1.0;
        r.min_months_of_stock = 0.9;
    })
}

pub fn mock_request_draft_requisition2() -> RequisitionRow {
    inline_init(|r: &mut RequisitionRow| {
        r.id = "mock_request_draft_requisition2".to_owned();
        r.requisition_number = 2;
        r.name_id = "name_a".to_owned();
        r.store_id = "store_a".to_owned();
        r.r#type = RequisitionRowType::Request;
        r.status = RequisitionRowStatus::Draft;
        r.created_datetime = NaiveDate::from_ymd(2021, 01, 01).and_hms(0, 0, 0);
        r.max_months_of_stock = 1.0;
        r.min_months_of_stock = 0.9;
    })
}
