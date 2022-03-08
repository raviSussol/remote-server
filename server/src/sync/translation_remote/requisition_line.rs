use repository::{
    schema::{ChangelogRow, ChangelogTableName, RemoteSyncBufferRow, RequisitionLineRow},
    RequisitionLineRowRepository, StorageConnection,
};

use serde::{Deserialize, Serialize};
use util::constants::NUMBER_OF_DAYS_IN_A_MONTH;

use crate::sync::SyncTranslationError;

use super::{
    empty_str_as_option,
    pull::{IntegrationRecord, IntegrationUpsertRecord, RemotePullTranslation},
    push::{to_push_translation_error, PushUpsertRecord, RemotePushUpsertTranslation},
    TRANSLATION_RECORD_REQUISITION_LINE,
};

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, PartialEq)]
pub struct LegacyRequisitionLineRow {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "requisition_ID")]
    pub requisition_id: String,
    #[serde(rename = "item_ID")]
    pub item_id: String,

    #[serde(rename = "Cust_stock_order")]
    pub requested_quantity: i32,
    pub suggested_quantity: i32,
    #[serde(rename = "actualQuan")]
    pub supply_quantity: i32,
    #[serde(rename = "stock_on_hand")]
    pub available_stock_on_hand: i32,
    // average_monthly_consumption: daily_usage * NUMBER_OF_DAYS_IN_A_MONTH
    pub daily_usage: f64,

    #[serde(deserialize_with = "empty_str_as_option")]
    pub comment: Option<String>,
}

pub struct RequisitionLineTranslation {}
impl RemotePullTranslation for RequisitionLineTranslation {
    fn try_translate_pull(
        &self,
        _: &StorageConnection,
        sync_record: &RemoteSyncBufferRow,
    ) -> Result<Option<IntegrationRecord>, SyncTranslationError> {
        let table_name = TRANSLATION_RECORD_REQUISITION_LINE;

        if sync_record.table_name != table_name {
            return Ok(None);
        }

        let data = serde_json::from_str::<LegacyRequisitionLineRow>(&sync_record.data).map_err(
            |source| SyncTranslationError {
                table_name,
                source: source.into(),
                record: sync_record.data.clone(),
            },
        )?;

        let LegacyRequisitionLineRow {
            id,
            requisition_id,
            item_id,
            requested_quantity,
            suggested_quantity,
            supply_quantity,
            available_stock_on_hand,
            daily_usage,
            comment,
        } = data;

        Ok(Some(IntegrationRecord::from_upsert(
            IntegrationUpsertRecord::RequisitionLine(RequisitionLineRow {
                // Simple
                id,
                requisition_id,
                item_id,
                requested_quantity,
                suggested_quantity,
                supply_quantity,
                available_stock_on_hand,
                comment,
                // Complex
                average_monthly_consumption: (daily_usage * NUMBER_OF_DAYS_IN_A_MONTH) as i32,
            }),
        )))
    }
}

impl RemotePushUpsertTranslation for RequisitionLineTranslation {
    fn try_translate_push(
        &self,
        connection: &StorageConnection,
        changelog: &ChangelogRow,
    ) -> Result<Option<Vec<PushUpsertRecord>>, SyncTranslationError> {
        if changelog.table_name != ChangelogTableName::RequisitionLine {
            return Ok(None);
        }
        let table_name = TRANSLATION_RECORD_REQUISITION_LINE;

        let RequisitionLineRow {
            id,
            requisition_id,
            item_id,
            requested_quantity,
            suggested_quantity,
            supply_quantity,
            available_stock_on_hand,
            average_monthly_consumption,
            comment,
        } = RequisitionLineRowRepository::new(connection)
            .find_one_by_id(&changelog.row_id)
            .map_err(|err| to_push_translation_error(table_name, err.into(), changelog))?
            .ok_or(to_push_translation_error(
                table_name,
                anyhow::Error::msg(format!(
                    "Requisition line row not found: {}",
                    changelog.row_id
                )),
                changelog,
            ))?;

        let legacy_row = LegacyRequisitionLineRow {
            // Simple
            id: id.clone(),
            requisition_id,
            item_id,
            requested_quantity,
            suggested_quantity,
            supply_quantity,
            available_stock_on_hand,
            comment,
            // Complex
            daily_usage: average_monthly_consumption as f64 / NUMBER_OF_DAYS_IN_A_MONTH,
        };

        Ok(Some(vec![PushUpsertRecord {
            sync_id: changelog.id,
            // TODO:
            store_id: None,
            table_name,
            record_id: id,
            data: serde_json::to_value(&legacy_row)
                .map_err(|err| to_push_translation_error(table_name, err.into(), changelog))?,
        }]))
    }
}
