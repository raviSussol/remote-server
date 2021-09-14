use super::SyncRecord;

use crate::database::schema::MasterListLineRow;

use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct LegacyListMasterLineRow {
    ID: String,
    item_master_ID: String,
    item_ID: String,
}

impl LegacyListMasterLineRow {
    pub fn try_translate(sync_record: &SyncRecord) -> Result<Option<MasterListLineRow>, String> {
        if sync_record.record_type != "list_master_line" {
            return Ok(None);
        }
        let data = serde_json::from_str::<LegacyListMasterLineRow>(&sync_record.data)
            .map_err(|_| "Deserialization Error".to_string())?;
        Ok(Some(MasterListLineRow {
            id: data.ID,
            item_id: data.item_ID,
            master_list_id: data.item_master_ID,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::util::sync::translation::{
        list_master_line::LegacyListMasterLineRow,
        test_data::{master_list_line::get_test_master_list_line_records, TestSyncDataRecord},
    };

    #[test]
    fn test_list_master_line_translation() {
        for record in get_test_master_list_line_records() {
            match record.translated_record {
                TestSyncDataRecord::MasterListLine(translated_record) => {
                    assert_eq!(
                        LegacyListMasterLineRow::try_translate(&record.sync_record).unwrap(),
                        translated_record,
                        "{}",
                        record.identifier
                    )
                }
                _ => panic!("Testing wrong record type {:#?}", record.translated_record),
            }
        }
    }
}
