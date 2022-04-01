use chrono::NaiveDate;
use util::inline_init;

use crate::schema::{
    InvoiceLineRow, InvoiceLineRowType, InvoiceRow, InvoiceRowStatus, InvoiceRowType, ItemRow,
    ItemRowType,
};

use super::MockData;

fn mock_outbound_shipment_line_no_stock_line() -> InvoiceLineRow {
    InvoiceLineRow {
        id: String::from("outbound_shipment_line_no_stock_line"),
        invoice_id: String::from("outbound_shipment_invalid_stock_line"),
        item_id: String::from("item_with_no_stock_line"),
        location_id: None,
        item_name: String::from("item_b"),
        item_code: String::from("item_b"),
        stock_line_id: None,
        batch: Some(String::from("item_a_line_a")),
        expiry_date: Some(NaiveDate::from_ymd(2020, 8, 2)),
        pack_size: 1,
        cost_price_per_pack: 0.0,
        sell_price_per_pack: 0.0,
        total_before_tax: 2.0,
        total_after_tax: 2.0,
        tax: None,
        r#type: InvoiceLineRowType::StockOut,
        number_of_packs: 1,
        note: None,
    }
}

fn mock_item_with_no_stock_line() -> ItemRow {
    inline_init(|r: &mut ItemRow| {
        r.id = String::from("item_with_no_stock_line");
        r.name = String::from("Item with no stock line");
        r.code = String::from("code");
        r.r#type = ItemRowType::Stock;
    })
}

// invoice containing invoice lines without stock line
fn mock_outbound_shipment_invalid_stock_line() -> InvoiceRow {
    inline_init(|r: &mut InvoiceRow| {
        r.id = String::from("outbound_shipment_invalid_stock_line");
        r.name_id = String::from("name_store_a");
        r.store_id = String::from("store_c");
        r.invoice_number = 3;
        r.r#type = InvoiceRowType::OutboundShipment;
        r.status = InvoiceRowStatus::New;
        r.comment = Some("Sort comment test cA".to_owned());
        r.their_reference = Some(String::from(""));
        r.created_datetime = NaiveDate::from_ymd(1970, 1, 6).and_hms_milli(15, 30, 0, 0);
    })
}

pub fn test_outbound_shipment_update_data() -> MockData {
    let mut data: MockData = Default::default();
    data.items.append(&mut vec![mock_item_with_no_stock_line()]);
    data.invoices
        .append(&mut vec![mock_outbound_shipment_invalid_stock_line()]);
    data.invoice_lines
        .append(&mut vec![mock_outbound_shipment_line_no_stock_line()]);
    data
}
