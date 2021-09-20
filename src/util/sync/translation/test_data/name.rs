use crate::{
    database::schema::{CentralSyncBufferRow, NameRow},
    util::sync::translation::test_data::{TestSyncDataRecord, TestSyncRecord},
};

const NAME_1: (&'static str, &'static str) = (
    "1FB32324AF8049248D929CFB35F255BA",
    r#"{
    "ID": "1FB32324AF8049248D929CFB35F255BA",
    "name": "General",
    "fax": "",
    "phone": "",
    "customer": true,
    "bill_address1": "",
    "bill_address2": "",
    "supplier": true,
    "charge code": "GEN",
    "margin": 0,
    "comment": "",
    "currency_ID": "",
    "country": "",
    "freightfac": 0,
    "email": "",
    "custom1": "",
    "code": "GEN",
    "last": "",
    "first": "",
    "title": "",
    "female": false,
    "date_of_birth": "0000-00-00",
    "overpayment": 0,
    "group_ID": "",
    "hold": false,
    "ship_address1": "",
    "ship_address2": "",
    "url": "",
    "barcode": "",
    "postal_address1": "",
    "postal_address2": "",
    "category1_ID": "",
    "region_ID": "",
    "type": "store",
    "price_category": "",
    "flag": "",
    "manufacturer": false,
    "print_invoice_alphabetical": false,
    "custom2": "",
    "custom3": "",
    "default_order_days": 0,
    "connection_type": 0,
    "PATIENT_PHOTO": "[object Picture]",
    "NEXT_OF_KIN_ID": "",
    "POBOX": "",
    "ZIP": 0,
    "middle": "",
    "preferred": false,
    "Blood_Group": "",
    "marital_status": "",
    "Benchmark": false,
    "next_of_kin_relative": "",
    "mother_id": "",
    "postal_address3": "",
    "postal_address4": "",
    "bill_address3": "",
    "bill_address4": "",
    "ship_address3": "",
    "ship_address4": "",
    "ethnicity_ID": "",
    "occupation_ID": "",
    "religion_ID": "",
    "national_health_number": "",
    "Master_RTM_Supplier_Code": 0,
    "ordering_method": "",
    "donor": false,
    "latitude": 0,
    "longitude": 0,
    "Master_RTM_Supplier_name": "",
    "category2_ID": "",
    "category3_ID": "",
    "category4_ID": "",
    "category5_ID": "",
    "category6_ID": "",
    "bill_address5": "",
    "bill_postal_zip_code": "",
    "postal_address5": "",
    "postal_zip_code": "",
    "ship_address5": "",
    "ship_postal_zip_code": "",
    "supplying_store_id": "",
    "license_number": "",
    "license_expiry": "0000-00-00",
    "has_current_license": false,
    "custom_data": null,
    "maximum_credit": 0,
    "nationality_ID": "",
    "created_date": "0000-00-00"
}"#,
);

const NAME_2: (&'static str, &'static str) = (
    "9EDD3F83C3D64C22A3CC9C98CF4967C4",
    r#"{
    "ID": "9EDD3F83C3D64C22A3CC9C98CF4967C4",
    "name": "Birch Store",
    "fax": "",
    "phone": "",
    "customer": true,
    "bill_address1": "234 Evil Street",
    "bill_address2": "Scotland",
    "supplier": false,
    "charge code": "SNA",
    "margin": 0,
    "comment": "",
    "currency_ID": "8009D512AC0E4FD78625E3C8273B0171",
    "country": "",
    "freightfac": 1,
    "email": "",
    "custom1": "",
    "code": "SNA",
    "last": "",
    "first": "",
    "title": "",
    "female": false,
    "date_of_birth": "0000-00-00",
    "overpayment": 0,
    "group_ID": "",
    "hold": false,
    "ship_address1": "",
    "ship_address2": "",
    "url": "",
    "barcode": "*SNA*",
    "postal_address1": "",
    "postal_address2": "",
    "category1_ID": "",
    "region_ID": "",
    "type": "facility",
    "price_category": "A",
    "flag": "",
    "manufacturer": false,
    "print_invoice_alphabetical": false,
    "custom2": "",
    "custom3": "",
    "default_order_days": 0,
    "connection_type": 0,
    "PATIENT_PHOTO": "[object Picture]",
    "NEXT_OF_KIN_ID": "",
    "POBOX": "",
    "ZIP": 0,
    "middle": "",
    "preferred": false,
    "Blood_Group": "",
    "marital_status": "",
    "Benchmark": false,
    "next_of_kin_relative": "",
    "mother_id": "",
    "postal_address3": "",
    "postal_address4": "",
    "bill_address3": "",
    "bill_address4": "",
    "ship_address3": "",
    "ship_address4": "",
    "ethnicity_ID": "",
    "occupation_ID": "",
    "religion_ID": "",
    "national_health_number": "",
    "Master_RTM_Supplier_Code": 0,
    "ordering_method": "sh",
    "donor": false,
    "latitude": 0,
    "longitude": 0,
    "Master_RTM_Supplier_name": "",
    "category2_ID": "",
    "category3_ID": "",
    "category4_ID": "",
    "category5_ID": "",
    "category6_ID": "",
    "bill_address5": "",
    "bill_postal_zip_code": "",
    "postal_address5": "",
    "postal_zip_code": "",
    "ship_address5": "",
    "ship_postal_zip_code": "",
    "supplying_store_id": "D77F67339BF8400886D009178F4962E1",
    "license_number": "",
    "license_expiry": "0000-00-00",
    "has_current_license": false,
    "custom_data": null,
    "maximum_credit": 0,
    "nationality_ID": "",
    "created_date": "0000-00-00"
}"#,
);

const NAME_3: (&'static str, &'static str) = (
    "CB929EB86530455AB0392277FAC3DBA4",
    r#"{
    "ID": "CB929EB86530455AB0392277FAC3DBA4",
    "name": "Birch Store 2",
    "fax": "",
    "phone": "",
    "customer": true,
    "bill_address1": "234 Evil Street",
    "bill_address2": "Scotland",
    "supplier": false,
    "charge code": "SNA",
    "margin": 0,
    "comment": "",
    "currency_ID": "8009D512AC0E4FD78625E3C8273B0171",
    "country": "",
    "freightfac": 1,
    "email": "",
    "custom1": "",
    "code": "SNA",
    "last": "",
    "first": "",
    "title": "",
    "female": false,
    "date_of_birth": "0000-00-00",
    "overpayment": 0,
    "group_ID": "",
    "hold": false,
    "ship_address1": "",
    "ship_address2": "",
    "url": "",
    "barcode": "*SNA*",
    "postal_address1": "",
    "postal_address2": "",
    "category1_ID": "",
    "region_ID": "",
    "type": "facility",
    "price_category": "A",
    "flag": "",
    "manufacturer": false,
    "print_invoice_alphabetical": false,
    "custom2": "",
    "custom3": "",
    "default_order_days": 0,
    "connection_type": 0,
    "PATIENT_PHOTO": "[object Picture]",
    "NEXT_OF_KIN_ID": "",
    "POBOX": "",
    "ZIP": 0,
    "middle": "",
    "preferred": false,
    "Blood_Group": "",
    "marital_status": "",
    "Benchmark": false,
    "next_of_kin_relative": "",
    "mother_id": "",
    "postal_address3": "",
    "postal_address4": "",
    "bill_address3": "",
    "bill_address4": "",
    "ship_address3": "",
    "ship_address4": "",
    "ethnicity_ID": "",
    "occupation_ID": "",
    "religion_ID": "",
    "national_health_number": "",
    "Master_RTM_Supplier_Code": 0,
    "ordering_method": "sh",
    "donor": false,
    "latitude": 0,
    "longitude": 0,
    "Master_RTM_Supplier_name": "",
    "category2_ID": "",
    "category3_ID": "",
    "category4_ID": "",
    "category5_ID": "",
    "category6_ID": "",
    "bill_address5": "",
    "bill_postal_zip_code": "",
    "postal_address5": "",
    "postal_zip_code": "",
    "ship_address5": "",
    "ship_postal_zip_code": "",
    "supplying_store_id": "D77F67339BF8400886D009178F4962E1",
    "license_number": "",
    "license_expiry": "0000-00-00",
    "has_current_license": false,
    "custom_data": null,
    "maximum_credit": 0,
    "nationality_ID": "",
    "created_date": "0000-00-00"
}"#,
);

const NAME_UPSERT_1: (&'static str, &'static str) = (
    "1FB32324AF8049248D929CFB35F255BA",
    r#"{
    "ID": "1FB32324AF8049248D929CFB35F255BA",
    "name": "General2",
    "fax": "",
    "phone": "",
    "customer": true,
    "bill_address1": "",
    "bill_address2": "",
    "supplier": true,
    "charge code": "GEN",
    "margin": 0,
    "comment": "",
    "currency_ID": "",
    "country": "",
    "freightfac": 0,
    "email": "",
    "custom1": "",
    "code": "GEN",
    "last": "",
    "first": "",
    "title": "",
    "female": false,
    "date_of_birth": "0000-00-00",
    "overpayment": 0,
    "group_ID": "",
    "hold": false,
    "ship_address1": "",
    "ship_address2": "",
    "url": "",
    "barcode": "",
    "postal_address1": "",
    "postal_address2": "",
    "category1_ID": "",
    "region_ID": "",
    "type": "store",
    "price_category": "",
    "flag": "",
    "manufacturer": false,
    "print_invoice_alphabetical": false,
    "custom2": "",
    "custom3": "",
    "default_order_days": 0,
    "connection_type": 0,
    "PATIENT_PHOTO": "[object Picture]",
    "NEXT_OF_KIN_ID": "",
    "POBOX": "",
    "ZIP": 0,
    "middle": "",
    "preferred": false,
    "Blood_Group": "",
    "marital_status": "",
    "Benchmark": false,
    "next_of_kin_relative": "",
    "mother_id": "",
    "postal_address3": "",
    "postal_address4": "",
    "bill_address3": "",
    "bill_address4": "",
    "ship_address3": "",
    "ship_address4": "",
    "ethnicity_ID": "",
    "occupation_ID": "",
    "religion_ID": "",
    "national_health_number": "",
    "Master_RTM_Supplier_Code": 0,
    "ordering_method": "",
    "donor": false,
    "latitude": 0,
    "longitude": 0,
    "Master_RTM_Supplier_name": "",
    "category2_ID": "",
    "category3_ID": "",
    "category4_ID": "",
    "category5_ID": "",
    "category6_ID": "",
    "bill_address5": "",
    "bill_postal_zip_code": "",
    "postal_address5": "",
    "postal_zip_code": "",
    "ship_address5": "",
    "ship_postal_zip_code": "",
    "supplying_store_id": "",
    "license_number": "",
    "license_expiry": "0000-00-00",
    "has_current_license": false,
    "custom_data": null,
    "maximum_credit": 0,
    "nationality_ID": "",
    "created_date": "0000-00-00"
}"#,
);

#[allow(dead_code)]
const RECORD_TYPE: &'static str = "name";
#[allow(dead_code)]
pub fn get_test_name_records() -> Vec<TestSyncRecord> {
    vec![
        TestSyncRecord {
            translated_record: TestSyncDataRecord::Name(Some(NameRow {
                id: NAME_1.0.to_owned(),
                name: "General".to_owned(),
                code: "GEN".to_owned(),
                is_supplier: true,
                is_customer: true,
            })),
            identifier: "General",
            central_sync_buffer_row: CentralSyncBufferRow {
                id: 200,
                table_name: RECORD_TYPE.to_owned(),
                record_id: NAME_1.0.to_owned(),
                data: NAME_1.1.to_owned(),
            },
        },
        TestSyncRecord {
            translated_record: TestSyncDataRecord::Name(Some(NameRow {
                id: NAME_2.0.to_owned(),
                name: "Birch Store".to_owned(),
                code: "SNA".to_owned(),
                is_customer: true,
                is_supplier: false,
            })),
            identifier: "Birch Store",
            central_sync_buffer_row: CentralSyncBufferRow {
                id: 201,
                table_name: RECORD_TYPE.to_owned(),
                record_id: NAME_2.0.to_owned(),
                data: NAME_2.1.to_owned(),
            },
        },
        TestSyncRecord {
            translated_record: TestSyncDataRecord::Name(Some(NameRow {
                id: NAME_3.0.to_owned(),
                name: "Birch Store 2".to_owned(),
                code: "SNA".to_owned(),
                is_customer: true,
                is_supplier: false,
            })),
            identifier: "Birch Store",
            central_sync_buffer_row: CentralSyncBufferRow {
                id: 202,
                table_name: RECORD_TYPE.to_owned(),
                record_id: NAME_3.0.to_owned(),
                data: NAME_3.1.to_owned(),
            },
        },
    ]
}
#[allow(dead_code)]
pub fn get_test_name_upsert_records() -> Vec<TestSyncRecord> {
    vec![TestSyncRecord {
        translated_record: TestSyncDataRecord::Name(Some(NameRow {
            id: NAME_UPSERT_1.0.to_owned(),
            name: "General2".to_owned(),
            code: "GEN".to_owned(),
            is_customer: true,
            is_supplier: true,
        })),
        identifier: "General2",
        central_sync_buffer_row: CentralSyncBufferRow {
            id: 250,
            table_name: RECORD_TYPE.to_owned(),
            record_id: NAME_UPSERT_1.0.to_owned(),
            data: NAME_UPSERT_1.1.to_owned(),
        },
    }]
}
