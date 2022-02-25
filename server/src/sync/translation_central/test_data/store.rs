use crate::sync::translation_central::test_data::{TestSyncDataRecord, TestSyncRecord};
use repository::schema::{CentralSyncBufferRow, StoreRow};

const STORE_1: (&'static str, &'static str) = (
    "4E27CEB263354EB7B1B33CEA8F7884D8",
    r#"{
    "ID": "4E27CEB263354EB7B1B33CEA8F7884D8",
    "name": "General",
    "code": "GEN",
    "name_ID": "1FB32324AF8049248D929CFB35F255BA",
    "mwks_export_mode": "",
    "IS_HIS": false,
    "sort_issues_by_status_spare": false,
    "disabled": false,
    "responsible_user_ID": "",
    "organisation_name": "",
    "address_1": "",
    "address_2": "",
    "logo": "[object Picture]",
    "sync_id_remote_site": 1,
    "address_3": "",
    "address_4": "",
    "address_5": "",
    "postal_zip_code": "",
    "store_mode": "store",
    "phone": "",
    "tags": "",
    "spare_user_1": "",
    "spare_user_2": "",
    "spare_user_3": "",
    "spare_user_4": "",
    "spare_user_5": "",
    "spare_user_6": "",
    "spare_user_7": "",
    "spare_user_8": "",
    "spare_user_9": "",
    "spare_user_10": "",
    "spare_user_11": "",
    "spare_user_12": "",
    "spare_user_13": "",
    "spare_user_14": "",
    "spare_user_15": "",
    "spare_user_16": "",
    "custom_data": null,
    "created_date": "2021-09-03"
}"#,
);

const STORE_2: (&'static str, &'static str) = (
    "9EDD3F83C3D64C22A3CC9C98CF4967C4",
    r#"{
    "ID": "9EDD3F83C3D64C22A3CC9C98CF4967C4",
    "name": "Drug Registration",
    "code": "DRG",
    "name_ID": "9A3F71AA4C6D48649ADBC4B2966C5B9D",
    "mwks_export_mode": "",
    "IS_HIS": false,
    "sort_issues_by_status_spare": false,
    "disabled": false,
    "responsible_user_ID": "",
    "organisation_name": "",
    "address_1": "",
    "address_2": "",
    "logo": "[object Picture]",
    "sync_id_remote_site": 1,
    "address_3": "",
    "address_4": "",
    "address_5": "",
    "postal_zip_code": "",
    "store_mode": "drug_registration",
    "phone": "",
    "tags": "",
    "spare_user_1": "",
    "spare_user_2": "",
    "spare_user_3": "",
    "spare_user_4": "",
    "spare_user_5": "",
    "spare_user_6": "",
    "spare_user_7": "",
    "spare_user_8": "",
    "spare_user_9": "",
    "spare_user_10": "",
    "spare_user_11": "",
    "spare_user_12": "",
    "spare_user_13": "",
    "spare_user_14": "",
    "spare_user_15": "",
    "spare_user_16": "",
    "custom_data": null,
    "created_date": "0000-00-00"
}"#,
);
const STORE_3: (&'static str, &'static str) = (
    "9A3F71AA4C6D48649ADBC4B2966C5B9D",
    r#"{
    "ID": "9A3F71AA4C6D48649ADBC4B2966C5B9D",
    "name": "Supervisor- All stores",
    "code": "SM",
    "name_ID": "",
    "mwks_export_mode": "",
    "IS_HIS": false,
    "sort_issues_by_status_spare": false,
    "disabled": false,
    "responsible_user_ID": "",
    "organisation_name": "",
    "address_1": "",
    "address_2": "",
    "logo": "[object Picture]",
    "sync_id_remote_site": 1,
    "address_3": "",
    "address_4": "",
    "address_5": "",
    "postal_zip_code": "",
    "store_mode": "supervisor",
    "phone": "",
    "tags": "",
    "spare_user_1": "",
    "spare_user_2": "",
    "spare_user_3": "",
    "spare_user_4": "",
    "spare_user_5": "",
    "spare_user_6": "",
    "spare_user_7": "",
    "spare_user_8": "",
    "spare_user_9": "",
    "spare_user_10": "",
    "spare_user_11": "",
    "spare_user_12": "",
    "spare_user_13": "",
    "spare_user_14": "",
    "spare_user_15": "",
    "spare_user_16": "",
    "custom_data": null,
    "created_date": "0000-00-00"
}"#,
);

const STORE_4: (&'static str, &'static str) = (
    "2CD38EF518764ED79258961101100C3D",
    r#"{
    "ID": "2CD38EF518764ED79258961101100C3D",
    "name": "Hospital Info System",
    "code": "HIS",
    "name_ID": "",
    "mwks_export_mode": "",
    "IS_HIS": true,
    "sort_issues_by_status_spare": false,
    "disabled": false,
    "responsible_user_ID": "",
    "organisation_name": "",
    "address_1": "",
    "address_2": "",
    "logo": "[object Picture]",
    "sync_id_remote_site": 1,
    "address_3": "",
    "address_4": "",
    "address_5": "",
    "postal_zip_code": "",
    "store_mode": "his",
    "phone": "",
    "tags": "",
    "spare_user_1": "",
    "spare_user_2": "",
    "spare_user_3": "",
    "spare_user_4": "",
    "spare_user_5": "",
    "spare_user_6": "",
    "spare_user_7": "",
    "spare_user_8": "",
    "spare_user_9": "",
    "spare_user_10": "",
    "spare_user_11": "",
    "spare_user_12": "",
    "spare_user_13": "",
    "spare_user_14": "",
    "spare_user_15": "",
    "spare_user_16": "",
    "custom_data": null,
    "created_date": "2021-09-03"
}"#,
);

#[allow(dead_code)]
const RECORD_TYPE: &'static str = "store";
#[allow(dead_code)]
pub fn get_test_store_records() -> Vec<TestSyncRecord> {
    vec![
        TestSyncRecord {
            translated_record: TestSyncDataRecord::Store(Some(StoreRow {
                id: STORE_1.0.to_owned(),
                name_id: "1FB32324AF8049248D929CFB35F255BA".to_owned(),
                code: "GEN".to_owned(),
            })),
            identifier: "General",
            central_sync_buffer_row: CentralSyncBufferRow {
                id: 10,
                table_name: RECORD_TYPE.to_owned(),
                record_id: STORE_1.0.to_owned(),
                data: STORE_1.1.to_owned(),
            },
        },
        TestSyncRecord {
            translated_record: TestSyncDataRecord::Store(None),
            identifier: "Drug Registration",
            central_sync_buffer_row: CentralSyncBufferRow {
                id: 11,
                table_name: RECORD_TYPE.to_owned(),
                record_id: STORE_2.0.to_owned(),
                data: STORE_2.1.to_owned(),
            },
        },
        TestSyncRecord {
            translated_record: TestSyncDataRecord::Store(None),
            identifier: "Supervisor- All stores",
            central_sync_buffer_row: CentralSyncBufferRow {
                id: 12,
                table_name: RECORD_TYPE.to_owned(),
                record_id: STORE_3.0.to_owned(),
                data: STORE_3.1.to_owned(),
            },
        },
        TestSyncRecord {
            translated_record: TestSyncDataRecord::Store(None),
            identifier: "Hospital Info System",
            central_sync_buffer_row: CentralSyncBufferRow {
                id: 13,
                table_name: RECORD_TYPE.to_owned(),
                record_id: STORE_4.0.to_owned(),
                data: STORE_4.1.to_owned(),
            },
        },
    ]
}