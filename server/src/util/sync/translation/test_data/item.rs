use crate::util::sync::translation::test_data::{TestSyncDataRecord, TestSyncRecord};
use repository::schema::{CentralSyncBufferRow, ItemRow};

const ITEM_1: (&'static str, &'static str) = (
    "8F252B5884B74888AAB73A0D42C09E7F",
    r#"{
    "ID": "8F252B5884B74888AAB73A0D42C09E7F",
    "item_name": "Non stock items",
    "start_of_year_date": "0000-00-00",
    "manufacture_method": "",
    "default_pack_size": 1,
    "dose_picture": "[object Picture]",
    "atc_category": "",
    "medication_purpose": "",
    "instructions": "",
    "user_field_7": false,
    "flags": "",
    "ddd_value": "",
    "code": "NSI",
    "other_names": "",
    "type_of": "non_stock",
    "price_editable": false,
    "margin": 0,
    "barcode_spare": "",
    "spare_ignore_for_orders": false,
    "sms_pack_size": 0,
    "expiry_date_mandatory": false,
    "volume_per_pack": 0,
    "department_ID": "",
    "weight": 0,
    "essential_drug_list": false,
    "catalogue_code": "",
    "indic_price": 0,
    "user_field_1": "",
    "spare_hold_for_issue": false,
    "builds_only": false,
    "reference_bom_quantity": 0,
    "use_bill_of_materials": false,
    "description": "",
    "spare_hold_for_receive": false,
    "Message": "",
    "interaction_group_ID": "",
    "spare_pack_to_one_on_receive": false,
    "cross_ref_item_ID": "",
    "strength": "",
    "user_field_4": false,
    "user_field_6": "",
    "spare_internal_analysis": 0,
    "user_field_2": "",
    "user_field_3": "",
    "ddd factor": 0,
    "account_stock_ID": "52923505A91447B9923BA34A4F332014",
    "account_purchases_ID": "330ACC81721C4126BD5DD6769466C5C4",
    "account_income_ID": "EF34ADD07C014AB8914E30CA2E3FEA8D",
    "unit_ID": "",
    "outer_pack_size": 0,
    "category_ID": "",
    "ABC_category": "",
    "warning_quantity": 0,
    "user_field_5": 0,
    "print_units_in_dis_labels": false,
    "volume_per_outer_pack": 0,
    "normal_stock": false,
    "critical_stock": false,
    "spare_non_stock": false,
    "non_stock_name_ID": "",
    "is_sync": false,
    "sms_code": "",
    "category2_ID": "",
    "category3_ID": "",
    "buy_price": 0,
    "VEN_category": "",
    "universalcodes_code": "",
    "universalcodes_name": "",
    "kit_data": null,
    "custom_data": null,
    "doses": 0,
    "is_vaccine": false,
    "restricted_location_type_ID": ""
}"#,
);

const ITEM_1_UPSERT: (&'static str, &'static str) = (
    "8F252B5884B74888AAB73A0D42C09E7F",
    r#"{
    "ID": "8F252B5884B74888AAB73A0D42C09E7F",
    "item_name": "Non stock items 2",
    "start_of_year_date": "0000-00-00",
    "manufacture_method": "",
    "default_pack_size": 1,
    "dose_picture": "[object Picture]",
    "atc_category": "",
    "medication_purpose": "",
    "instructions": "",
    "user_field_7": false,
    "flags": "",
    "ddd_value": "",
    "code": "NSI",
    "other_names": "",
    "type_of": "general",
    "price_editable": false,
    "margin": 0,
    "barcode_spare": "",
    "spare_ignore_for_orders": false,
    "sms_pack_size": 0,
    "expiry_date_mandatory": false,
    "volume_per_pack": 0,
    "department_ID": "",
    "weight": 0,
    "essential_drug_list": false,
    "catalogue_code": "",
    "indic_price": 0,
    "user_field_1": "",
    "spare_hold_for_issue": false,
    "builds_only": false,
    "reference_bom_quantity": 0,
    "use_bill_of_materials": false,
    "description": "",
    "spare_hold_for_receive": false,
    "Message": "",
    "interaction_group_ID": "",
    "spare_pack_to_one_on_receive": false,
    "cross_ref_item_ID": "",
    "strength": "",
    "user_field_4": false,
    "user_field_6": "",
    "spare_internal_analysis": 0,
    "user_field_2": "",
    "user_field_3": "",
    "ddd factor": 0,
    "account_stock_ID": "52923505A91447B9923BA34A4F332014",
    "account_purchases_ID": "330ACC81721C4126BD5DD6769466C5C4",
    "account_income_ID": "EF34ADD07C014AB8914E30CA2E3FEA8D",
    "unit_ID": "A02C91EB6C77400BA783C4CD7C565F29",
    "outer_pack_size": 0,
    "category_ID": "",
    "ABC_category": "",
    "warning_quantity": 0,
    "user_field_5": 0,
    "print_units_in_dis_labels": false,
    "volume_per_outer_pack": 0,
    "normal_stock": false,
    "critical_stock": false,
    "spare_non_stock": false,
    "non_stock_name_ID": "",
    "is_sync": false,
    "sms_code": "",
    "category2_ID": "",
    "category3_ID": "",
    "buy_price": 0,
    "VEN_category": "",
    "universalcodes_code": "",
    "universalcodes_name": "",
    "kit_data": null,
    "custom_data": null,
    "doses": 0,
    "is_vaccine": false,
    "restricted_location_type_ID": ""
}"#,
);

#[allow(dead_code)]
const RECORD_TYPE: &'static str = "item";
#[allow(dead_code)]
pub fn get_test_item_records() -> Vec<TestSyncRecord> {
    vec![TestSyncRecord {
        translated_record: TestSyncDataRecord::Item(Some(ItemRow {
            id: ITEM_1.0.to_owned(),
            name: "Non stock items".to_owned(),
            code: "NSI".to_owned(),
            unit_id: None,
        })),
        identifier: "Non stock items",
        central_sync_buffer_row: CentralSyncBufferRow {
            id: 300,
            table_name: RECORD_TYPE.to_owned(),
            record_id: ITEM_1.0.to_owned(),
            data: ITEM_1.1.to_owned(),
        },
    }]
}
#[allow(dead_code)]
pub fn get_test_item_upsert_records() -> Vec<TestSyncRecord> {
    vec![TestSyncRecord {
        translated_record: TestSyncDataRecord::Item(Some(ItemRow {
            id: ITEM_1_UPSERT.0.to_owned(),
            name: "Non stock items 2".to_owned(),
            code: "NSI".to_owned(),
            unit_id: Some("A02C91EB6C77400BA783C4CD7C565F29".to_owned()),
        })),
        identifier: "Non stock items 2",
        central_sync_buffer_row: CentralSyncBufferRow {
            id: 301,
            table_name: RECORD_TYPE.to_owned(),
            record_id: ITEM_1_UPSERT.0.to_owned(),
            data: ITEM_1_UPSERT.1.to_owned(),
        },
    }]
}
