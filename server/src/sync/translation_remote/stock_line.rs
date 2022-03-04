use chrono::NaiveDate;
use repository::{
    schema::{ChangelogRow, ChangelogTableName, RemoteSyncBufferRow, StockLineRow},
    StockLineRowRepository, StorageConnection,
};

use serde::{Deserialize, Serialize};

use crate::sync::SyncTranslationError;

use super::{
    empty_str_as_option,
    pull::{IntegrationRecord, IntegrationUpsertRecord, RemotePullTranslation},
    push::{to_push_translation_error, PushUpsertRecord, RemotePushUpsertTranslation},
    zero_date_as_option, TRANSLATION_RECORD_ITEM_LINE,
};

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
pub struct LegacyStockLineRow {
    pub ID: String,
    pub store_ID: String,
    pub item_ID: String,
    #[serde(deserialize_with = "empty_str_as_option")]
    pub batch: Option<String>,
    #[serde(deserialize_with = "zero_date_as_option")]
    pub expiry_date: Option<NaiveDate>,
    pub hold: bool,
    #[serde(deserialize_with = "empty_str_as_option")]
    pub location_ID: Option<String>,
    pub pack_size: i32,
    pub available: i32,
    pub quantity: i32,
    pub cost_price: f64,
    pub sell_price: f64,
    pub note: Option<String>,
}

pub struct StockLineTranslation {}
impl RemotePullTranslation for StockLineTranslation {
    fn try_translate_pull(
        &self,
        _: &StorageConnection,
        sync_record: &RemoteSyncBufferRow,
    ) -> Result<Option<IntegrationRecord>, SyncTranslationError> {
        let table_name = TRANSLATION_RECORD_ITEM_LINE;

        if sync_record.table_name != table_name {
            return Ok(None);
        }

        let data =
            serde_json::from_str::<LegacyStockLineRow>(&sync_record.data).map_err(|source| {
                SyncTranslationError {
                    table_name,
                    source: source.into(),
                    record: sync_record.data.clone(),
                }
            })?;

        Ok(Some(IntegrationRecord::from_upsert(
            IntegrationUpsertRecord::StockLine(StockLineRow {
                id: data.ID,
                store_id: data.store_ID,
                item_id: data.item_ID,
                location_id: data.location_ID,
                batch: data.batch,
                pack_size: data.pack_size,
                cost_price_per_pack: data.cost_price,
                sell_price_per_pack: data.sell_price,
                available_number_of_packs: data.available,
                total_number_of_packs: data.quantity,
                expiry_date: data.expiry_date,
                on_hold: data.hold,
                note: data.note,
            }),
        )))
    }
}

impl RemotePushUpsertTranslation for StockLineTranslation {
    fn try_translate_push(
        &self,
        connection: &StorageConnection,
        changelog: &ChangelogRow,
    ) -> Result<Option<Vec<PushUpsertRecord>>, SyncTranslationError> {
        if changelog.table_name != ChangelogTableName::StockLine {
            return Ok(None);
        }
        let table_name = TRANSLATION_RECORD_ITEM_LINE;

        let StockLineRow {
            id,
            item_id,
            store_id,
            location_id,
            batch,
            pack_size,
            cost_price_per_pack,
            sell_price_per_pack,
            available_number_of_packs,
            total_number_of_packs,
            expiry_date,
            on_hold,
            note,
        } = StockLineRowRepository::new(connection)
            .find_one_by_id(&changelog.row_id)
            .map_err(|err| to_push_translation_error(table_name, err.into(), changelog))?;

        let legacy_row = LegacyStockLineRow {
            ID: id.clone(),
            store_ID: store_id.clone(),
            item_ID: item_id,
            batch,
            expiry_date,
            hold: on_hold,
            location_ID: location_id,
            pack_size,
            available: available_number_of_packs,
            quantity: total_number_of_packs,
            cost_price: cost_price_per_pack,
            sell_price: sell_price_per_pack,
            note,
        };

        Ok(Some(vec![PushUpsertRecord {
            sync_id: changelog.id,
            store_id: Some(store_id),
            table_name,
            record_id: id,
            data: serde_json::to_value(&legacy_row)
                .map_err(|err| to_push_translation_error(table_name, err.into(), changelog))?,
        }]))
    }
}
