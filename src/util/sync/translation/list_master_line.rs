use crate::database::schema::{CentralSyncBufferRow, MasterListLineRow};

use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct LegacyListMasterLineRow {
    ID: String,
    item_master_ID: String,
    item_ID: String,
}

impl LegacyListMasterLineRow {
    pub fn try_translate(
        sync_record: &CentralSyncBufferRow,
    ) -> Result<Option<MasterListLineRow>, serde_json::Error> {
        if sync_record.table_name != "list_master_line" {
            return Ok(None);
        }
        let data = serde_json::from_str::<LegacyListMasterLineRow>(&sync_record.data)?;

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
                        LegacyListMasterLineRow::try_translate(&record.central_sync_buffer_row)
                            .unwrap(),
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
