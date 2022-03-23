use chrono::NaiveDate;
use repository::{
    schema::{
        ChangelogRow, ChangelogTableName, InvoiceLineRow, InvoiceLineRowType, RemoteSyncBufferRow,
    },
    InvoiceLineRowRepository, ItemRepository, StorageConnection,
};

use serde::{Deserialize, Serialize};

use crate::sync::SyncTranslationError;

use super::{
    date_option_to_isostring, empty_str_as_option,
    pull::{IntegrationRecord, IntegrationUpsertRecord, RemotePullTranslation},
    push::{to_push_translation_error, PushUpsertRecord, RemotePushUpsertTranslation},
    zero_date_as_option, TRANSLATION_RECORD_TRANS_LINE,
};

#[derive(Deserialize, Serialize, Debug)]
pub enum LegacyTransLineType {
    #[serde(rename = "stock_in")]
    StockIn,
    #[serde(rename = "stock_out")]
    StockOut,
    #[serde(rename = "placeholder")]
    Placeholder,
    #[serde(rename = "service")]
    Service,
    #[serde(rename = "non_stock")]
    NonStock,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
pub struct LegacyTransLineRow {
    pub ID: String,
    pub transaction_ID: String,
    pub item_ID: String,
    pub item_name: String,
    // stock line id
    #[serde(deserialize_with = "empty_str_as_option")]
    pub item_line_ID: Option<String>,
    #[serde(deserialize_with = "empty_str_as_option")]
    pub location_ID: Option<String>,
    #[serde(deserialize_with = "empty_str_as_option")]
    pub batch: Option<String>,
    #[serde(deserialize_with = "zero_date_as_option")]
    #[serde(serialize_with = "date_option_to_isostring")]
    pub expiry_date: Option<NaiveDate>,
    pub pack_size: i32,
    pub cost_price: f64,
    pub sell_price: f64,
    #[serde(rename = "type")]
    pub _type: LegacyTransLineType,
    // number of packs
    pub quantity: i32,
    #[serde(deserialize_with = "empty_str_as_option")]
    pub note: Option<String>,

    #[serde(rename = "om_item_code")]
    #[serde(deserialize_with = "empty_str_as_option")]
    #[serde(default)]
    pub item_code: Option<String>,
    #[serde(rename = "om_tax")]
    pub tax: Option<f64>,
    #[serde(rename = "om_total_before_tax")]
    pub total_before_tax: Option<f64>,
    #[serde(rename = "om_total_after_tax")]
    pub total_after_tax: Option<f64>,
}

pub struct InvoiceLineTranslation {}
impl RemotePullTranslation for InvoiceLineTranslation {
    fn try_translate_pull(
        &self,
        connection: &StorageConnection,
        sync_record: &RemoteSyncBufferRow,
    ) -> Result<Option<IntegrationRecord>, SyncTranslationError> {
        let table_name = TRANSLATION_RECORD_TRANS_LINE;
        if sync_record.table_name != table_name {
            return Ok(None);
        }

        let data =
            serde_json::from_str::<LegacyTransLineRow>(&sync_record.data).map_err(|source| {
                SyncTranslationError {
                    table_name,
                    source: source.into(),
                    record: sync_record.data.clone(),
                }
            })?;

        let line_type = to_invoice_line_type(&data._type).ok_or(SyncTranslationError {
            table_name,
            source: anyhow::Error::msg(format!("Unsupported trans_line type: {:?}", data._type)),
            record: sync_record.data.clone(),
        })?;
        let total = total(&data);
        let item_code = match data.item_code {
            Some(item_code) => item_code,
            None => {
                let item = match ItemRepository::new(connection)
                    .find_one_by_id(&data.item_ID)
                    .map_err(|source| SyncTranslationError {
                        table_name,
                        source: source.into(),
                        record: sync_record.data.clone(),
                    })? {
                    Some(item) => item,
                    None => {
                        return Err(SyncTranslationError {
                            table_name,
                            source: anyhow::Error::msg(format!(
                                "Failed to get item: {}",
                                data.item_ID
                            )),
                            record: sync_record.data.clone(),
                        })
                    }
                };
                item.code
            }
        };
        Ok(Some(IntegrationRecord::from_upsert(
            IntegrationUpsertRecord::InvoiceLine(InvoiceLineRow {
                id: data.ID,
                invoice_id: data.transaction_ID,
                item_id: data.item_ID,
                item_name: data.item_name,
                item_code,
                stock_line_id: data.item_line_ID,
                location_id: data.location_ID,
                batch: data.batch,
                expiry_date: data.expiry_date,
                pack_size: data.pack_size,
                cost_price_per_pack: data.cost_price,
                sell_price_per_pack: data.sell_price,
                total_before_tax: data.total_before_tax.unwrap_or(total),
                total_after_tax: data.total_after_tax.unwrap_or(total),
                tax: data.tax,
                r#type: line_type,
                number_of_packs: data.quantity / data.pack_size,
                note: data.note,
            }),
        )))
    }
}

fn total(data: &LegacyTransLineRow) -> f64 {
    match data._type {
        LegacyTransLineType::StockIn => data.cost_price * data.quantity as f64,
        LegacyTransLineType::StockOut => data.sell_price * data.quantity as f64,
        LegacyTransLineType::Placeholder => 0.0,
        LegacyTransLineType::Service => 0.0,
        LegacyTransLineType::NonStock => 0.0,
    }
}

fn to_invoice_line_type(_type: &LegacyTransLineType) -> Option<InvoiceLineRowType> {
    let invoice_line_type = match _type {
        LegacyTransLineType::StockIn => InvoiceLineRowType::StockIn,
        LegacyTransLineType::StockOut => InvoiceLineRowType::StockOut,
        LegacyTransLineType::Placeholder => InvoiceLineRowType::UnallocatedStock,
        LegacyTransLineType::Service => InvoiceLineRowType::Service,
        LegacyTransLineType::NonStock => return None,
    };
    Some(invoice_line_type)
}

impl RemotePushUpsertTranslation for InvoiceLineTranslation {
    fn try_translate_push(
        &self,
        connection: &StorageConnection,
        changelog: &ChangelogRow,
    ) -> Result<Option<Vec<PushUpsertRecord>>, SyncTranslationError> {
        if changelog.table_name != ChangelogTableName::InvoiceLine {
            return Ok(None);
        }
        let table_name = TRANSLATION_RECORD_TRANS_LINE;

        let InvoiceLineRow {
            id,
            invoice_id,
            item_id,
            item_name,
            item_code,
            stock_line_id,
            location_id,
            batch,
            expiry_date,
            pack_size,
            cost_price_per_pack,
            sell_price_per_pack,
            total_before_tax,
            total_after_tax,
            tax,
            r#type,
            number_of_packs,
            note,
        } = InvoiceLineRowRepository::new(connection)
            .find_one_by_id(&changelog.row_id)
            .map_err(|err| to_push_translation_error(table_name, err.into(), changelog))?;

        let legacy_row = LegacyTransLineRow {
            ID: id.clone(),
            transaction_ID: invoice_id,
            item_ID: item_id,
            item_name,
            item_line_ID: stock_line_id,
            location_ID: location_id,
            batch,
            expiry_date,
            pack_size,
            cost_price: cost_price_per_pack,
            sell_price: sell_price_per_pack,
            _type: to_legacy_invoice_line_type(&r#type),
            quantity: pack_size * number_of_packs,
            note,
            item_code: Some(item_code),
            tax,
            total_before_tax: Some(total_before_tax),
            total_after_tax: Some(total_after_tax),
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

fn to_legacy_invoice_line_type(_type: &InvoiceLineRowType) -> LegacyTransLineType {
    match _type {
        InvoiceLineRowType::StockIn => LegacyTransLineType::StockIn,
        InvoiceLineRowType::StockOut => LegacyTransLineType::StockOut,
        InvoiceLineRowType::UnallocatedStock => LegacyTransLineType::Placeholder,
        InvoiceLineRowType::Service => LegacyTransLineType::Service,
    }
}
