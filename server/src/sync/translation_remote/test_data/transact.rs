use chrono::{Duration, NaiveDate};
use repository::schema::{
    InvoiceRow, InvoiceRowStatus, InvoiceRowType, RemoteSyncBufferAction, RemoteSyncBufferRow,
};

use crate::sync::translation_remote::{
    test_data::TestSyncRecord, IntegrationRecord, IntegrationUpsertRecord,
    TRANSLATION_RECORD_TRANSACT,
};

const TRANSACT_1: (&'static str, &'static str) = (
    "12e889c0f0d211eb8dddb54df6d741bc",
    r#"{
      "Colour": 0,
      "Date_order_received": "0000-00-00",
      "Date_order_written": "2021-07-30",
      "ID": "12e889c0f0d211eb8dddb54df6d741bc",
      "amount_outstanding": 0,
      "arrival_date_actual": "0000-00-00",
      "arrival_date_estimated": "0000-00-00",
      "authorisationStatus": "",
      "budget_period_ID": "",
      "category2_ID": "",
      "category_ID": "",
      "comment": "",
      "confirm_date": "2021-07-30",
      "confirm_time": 47046,
      "contact_id": "",
      "currency_ID": "8009D512AC0E4FD78625E3C8273B0171",
      "currency_rate": 1,
      "custom_data": null,
      "diagnosis_ID": "",
      "donor_default_id": "",
      "encounter_id": "",
      "entry_date": "2021-07-30",
      "entry_time": 47046,
      "export_batch": 0,
      "foreign_currency_total": 0,
      "goodsReceivedConfirmation": null,
      "goods_received_ID": "",
      "hold": false,
      "insuranceDiscountAmount": 0,
      "insuranceDiscountRate": 0,
      "internalData": null,
      "invoice_num": 1,
      "invoice_printed_date": "0000-00-00",
      "is_authorised": false,
      "is_cancellation": false,
      "lastModifiedAt": 1627607293,
      "linked_goods_received_ID": "",
      "linked_transaction_id": "",
      "local_charge_distributed": 0,
      "mode": "store",
      "mwks_sequence_num": 0,
      "nameInsuranceJoinID": "",
      "name_ID": "name_store_a",
      "number_of_cartons": 0,
      "optionID": "",
      "original_PO_ID": "",
      "paymentTypeID": "",
      "pickslip_printed_date": "0000-00-00",
      "prescriber_ID": "",
      "requisition_ID": "",
      "responsible_officer_ID": "",
      "service_descrip": "",
      "service_price": 0,
      "ship_date": "0000-00-00",
      "ship_method_ID": "",
      "ship_method_comment": "",
      "status": "cn",
      "store_ID": "store_a",
      "subtotal": 0,
      "supplier_charge_fc": 0,
      "tax": 0,
      "their_ref": "",
      "total": 0,
      "type": "si",
      "user1": "",
      "user2": "",
      "user3": "",
      "user4": "",
      "user_ID": "",
      "wardID": "",
      "waybill_number": ""
  }"#,
);

const TRANSACT_2: (&'static str, &'static str) = (
    "7c860d40f3f111eb9647790fe8518386",
    r#"{
        "Colour": 1710361,
        "Date_order_received": "0000-00-00",
        "Date_order_written": "2021-08-03",
        "ID": "7c860d40f3f111eb9647790fe8518386",
        "amount_outstanding": 0,
        "arrival_date_actual": "0000-00-00",
        "arrival_date_estimated": "0000-00-00",
        "authorisationStatus": "",
        "budget_period_ID": "",
        "category2_ID": "",
        "category_ID": "",
        "comment": "",
        "confirm_date": "0000-00-00",
        "confirm_time": 44806,
        "contact_id": "",
        "currency_ID": "8009D512AC0E4FD78625E3C8273B0171",
        "currency_rate": 1,
        "custom_data": null,
        "diagnosis_ID": "",
        "donor_default_id": "",
        "encounter_id": "",
        "entry_date": "2021-08-03",
        "entry_time": 44806,
        "export_batch": 0,
        "foreign_currency_total": 0,
        "goodsReceivedConfirmation": null,
        "goods_received_ID": "",
        "hold": false,
        "insuranceDiscountAmount": 0,
        "insuranceDiscountRate": 0,
        "internalData": null,
        "invoice_num": 4,
        "invoice_printed_date": "0000-00-00",
        "is_authorised": false,
        "is_cancellation": false,
        "lastModifiedAt": 1627959832,
        "linked_goods_received_ID": "",
        "linked_transaction_id": "",
        "local_charge_distributed": 0,
        "mode": "dispensary",
        "mwks_sequence_num": 0,
        "nameInsuranceJoinID": "",
        "name_ID": "name_store_b",
        "number_of_cartons": 0,
        "optionID": "",
        "original_PO_ID": "",
        "paymentTypeID": "",
        "pickslip_printed_date": "0000-00-00",
        "prescriber_ID": "",
        "requisition_ID": "",
        "responsible_officer_ID": "",
        "service_descrip": "",
        "service_price": 0,
        "ship_date": "0000-00-00",
        "ship_method_ID": "",
        "ship_method_comment": "",
        "status": "fn",
        "store_ID": "store_b",
        "subtotal": 0,
        "supplier_charge_fc": 0,
        "tax": 0,
        "their_ref": "",
        "total": 0,
        "type": "ci",
        "user1": "",
        "user2": "",
        "user3": "",
        "user4": "",
        "user_ID": "0763E2E3053D4C478E1E6B6B03FEC207",
        "wardID": "",
        "waybill_number": ""
    }"#,
);

#[allow(dead_code)]
pub fn get_test_transact_records() -> Vec<TestSyncRecord> {
    vec![
        TestSyncRecord {
            translated_record: Some(IntegrationRecord::from_upsert(
                IntegrationUpsertRecord::Shipment(InvoiceRow {
                    id: TRANSACT_1.0.to_string(),
                    store_id: "store_a".to_string(),
                    name_id: "name_store_a".to_string(),
                    name_store_id: None,
                    invoice_number: 1,
                    r#type: InvoiceRowType::InboundShipment,
                    status: InvoiceRowStatus::Picked,
                    on_hold: false,
                    comment: None,
                    their_reference: None,
                    created_datetime: NaiveDate::from_ymd(2021, 7, 30).and_hms(0, 0, 0)
                        + Duration::seconds(47046),
                    allocated_datetime: None,
                    picked_datetime: None,
                    shipped_datetime: None,
                    delivered_datetime: None,
                    verified_datetime: Some(
                        NaiveDate::from_ymd(2021, 7, 30).and_hms(0, 0, 0)
                            + Duration::seconds(47046),
                    ),
                    colour: Some("#000000".to_string()),
                    requisition_id: None,
                    linked_invoice_id: None,
                }),
            )),
            identifier: "Transact 1",
            remote_sync_buffer_row: RemoteSyncBufferRow {
                id: "Transact_10".to_string(),
                table_name: TRANSLATION_RECORD_TRANSACT.to_string(),
                record_id: TRANSACT_1.0.to_string(),
                data: TRANSACT_1.1.to_string(),
                action: RemoteSyncBufferAction::Update,
            },
        },
        TestSyncRecord {
            translated_record: Some(IntegrationRecord::from_upsert(
                IntegrationUpsertRecord::Shipment(InvoiceRow {
                    id: TRANSACT_2.0.to_string(),
                    store_id: "store_b".to_string(),
                    name_id: "name_store_b".to_string(),
                    name_store_id: None,
                    invoice_number: 4,
                    r#type: InvoiceRowType::OutboundShipment,
                    status: InvoiceRowStatus::Verified,
                    on_hold: false,
                    comment: None,
                    their_reference: None,
                    created_datetime: NaiveDate::from_ymd(2021, 8, 3).and_hms(0, 0, 0)
                        + Duration::seconds(44806),
                    allocated_datetime: None,
                    picked_datetime: None,
                    shipped_datetime: None,
                    delivered_datetime: None,
                    verified_datetime: None,
                    colour: Some("#1A1919".to_string()),
                    requisition_id: None,
                    linked_invoice_id: None,
                }),
            )),
            identifier: "Transact 2",
            remote_sync_buffer_row: RemoteSyncBufferRow {
                id: "Transact_20".to_string(),
                table_name: TRANSLATION_RECORD_TRANSACT.to_string(),
                record_id: TRANSACT_2.0.to_string(),
                data: TRANSACT_2.1.to_string(),
                action: RemoteSyncBufferAction::Update,
            },
        },
    ]
}
