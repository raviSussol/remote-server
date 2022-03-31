use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use repository::{
    schema::{
        ChangelogRow, ChangelogTableName, InvoiceRow, InvoiceRowStatus, InvoiceRowType, NameRow,
        RemoteSyncBufferRow,
    },
    InvoiceRepository, NameRepository, StorageConnection, StoreRowRepository,
};

use serde::{Deserialize, Serialize};
use util::constants::INVENTORY_ADJUSTMENT_NAME_CODE;

use super::{
    date_and_time_to_datatime, date_from_date_time, date_option_to_isostring, date_to_isostring,
    empty_date_time_as_option, empty_str_as_option, naive_time,
    pull::{IntegrationRecord, IntegrationUpsertRecord, RemotePullTranslation},
    push::{PushUpsertRecord, RemotePushUpsertTranslation},
    zero_date_as_option, TRANSLATION_RECORD_TRANSACT,
};

#[derive(Deserialize, Serialize, Debug)]
pub enum LegacyTransactType {
    /// Supplier invoice
    #[serde(rename = "si")]
    Si,
    /// Customer invoice
    #[serde(rename = "ci")]
    Ci,
    // customer credit
    #[serde(rename = "cc")]
    Cc,
    // supplier credit
    #[serde(rename = "sc")]
    Sc,
    /// repack (A stock line is broken down into smaller pack sizes)
    #[serde(rename = "sr")]
    Sr,
    /// build- an internal transaction where you manufacture (build) items from raw materials in stock.
    #[serde(rename = "bu")]
    Bu,
    /// receipt (cash receipt) from a customer (a customer pays for invoices issued)
    #[serde(rename = "rc")]
    Rc,
    /// payment (cash payment) to a supplier
    #[serde(rename = "ps")]
    Ps,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum LegacyTransactStatus {
    /// new
    #[serde(rename = "nw")]
    Nw,
    /// suggested
    #[serde(rename = "sg")]
    Sg,
    /// confirmed
    #[serde(rename = "cn")]
    Cn,
    /// finalised
    #[serde(rename = "fn")]
    Fn,
    /// The order has been received over the internet (a “web” order), and it is currently being
    /// processed
    #[serde(rename = "wp")]
    Wp,
    /// The order has been received over the internet (a “web” order), and it is finalised
    #[serde(rename = "wf")]
    Wf,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
pub enum TransactMode {
    #[serde(rename = "store")]
    Store,
    #[serde(rename = "dispensary")]
    Dispensary,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatus {
    New,
    Allocated,
    Picked,
    Shipped,
    Delivered,
    Verified,
}
impl InvoiceStatus {
    fn to_domain(&self) -> InvoiceRowStatus {
        match self {
            InvoiceStatus::New => InvoiceRowStatus::New,
            InvoiceStatus::Allocated => InvoiceRowStatus::Allocated,
            InvoiceStatus::Picked => InvoiceRowStatus::Picked,
            InvoiceStatus::Shipped => InvoiceRowStatus::Shipped,
            InvoiceStatus::Delivered => InvoiceRowStatus::Delivered,
            InvoiceStatus::Verified => InvoiceRowStatus::Verified,
        }
    }

    fn from_domain(status: InvoiceRowStatus) -> Self {
        match status {
            InvoiceRowStatus::New => InvoiceStatus::New,
            InvoiceRowStatus::Allocated => InvoiceStatus::Allocated,
            InvoiceRowStatus::Picked => InvoiceStatus::Picked,
            InvoiceRowStatus::Shipped => InvoiceStatus::Shipped,
            InvoiceRowStatus::Delivered => InvoiceStatus::Delivered,
            InvoiceRowStatus::Verified => InvoiceStatus::Verified,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceType {
    OutboundShipment,
    InboundShipment,
    InventoryAdjustment,
}
impl InvoiceType {
    fn to_domain(&self) -> InvoiceRowType {
        match self {
            InvoiceType::OutboundShipment => InvoiceRowType::OutboundShipment,
            InvoiceType::InboundShipment => InvoiceRowType::InboundShipment,
            InvoiceType::InventoryAdjustment => InvoiceRowType::InventoryAdjustment,
        }
    }

    fn from_domain(status: InvoiceRowType) -> Self {
        match status {
            InvoiceRowType::OutboundShipment => InvoiceType::OutboundShipment,
            InvoiceRowType::InboundShipment => InvoiceType::InboundShipment,
            InvoiceRowType::InventoryAdjustment => InvoiceType::InventoryAdjustment,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
pub struct LegacyTransactRow {
    pub ID: String,

    pub name_ID: String,
    pub store_ID: String,
    pub invoice_num: i64,
    #[serde(rename = "type")]
    pub _type: LegacyTransactType,
    pub status: LegacyTransactStatus,
    #[serde(deserialize_with = "empty_str_as_option")]
    #[serde(rename = "user_ID")]
    pub user_id: Option<String>,
    pub hold: bool,
    #[serde(deserialize_with = "empty_str_as_option")]
    pub comment: Option<String>,
    #[serde(deserialize_with = "empty_str_as_option")]
    pub their_ref: Option<String>,

    #[serde(rename = "om_transport_reference")]
    #[serde(default)]
    #[serde(deserialize_with = "empty_str_as_option")]
    pub transport_reference: Option<String>,
    pub Colour: i32,
    #[serde(deserialize_with = "empty_str_as_option")]
    pub requisition_ID: Option<String>,
    #[serde(deserialize_with = "empty_str_as_option")]
    pub linked_transaction_id: Option<String>,

    /// creation time
    #[serde(serialize_with = "date_to_isostring")]
    pub entry_date: NaiveDate, // e.g. "2021-07-30",
    /// time in seconds
    #[serde(deserialize_with = "naive_time")]
    pub entry_time: NaiveTime, // e.g. 47046,
    /// shipped_datetime
    #[serde(deserialize_with = "zero_date_as_option")]
    #[serde(serialize_with = "date_option_to_isostring")]
    pub ship_date: Option<NaiveDate>, // "0000-00-00",
    /// delivered_datetime
    #[serde(deserialize_with = "zero_date_as_option")]
    #[serde(serialize_with = "date_option_to_isostring")]
    pub arrival_date_actual: Option<NaiveDate>,
    /// verified_datetime
    #[serde(deserialize_with = "zero_date_as_option")]
    #[serde(serialize_with = "date_option_to_isostring")]
    pub confirm_date: Option<NaiveDate>,
    #[serde(deserialize_with = "naive_time")]
    pub confirm_time: NaiveTime,

    pub mode: TransactMode,

    #[serde(rename = "om_created_datetime")]
    pub created_datetime: Option<NaiveDateTime>,

    #[serde(rename = "om_allocated_datetime")]
    #[serde(default)]
    #[serde(deserialize_with = "empty_date_time_as_option")]
    pub allocated_datetime: Option<NaiveDateTime>,

    #[serde(rename = "om_picked_datetime")]
    #[serde(default)]
    #[serde(deserialize_with = "empty_date_time_as_option")]
    pub picked_datetime: Option<NaiveDateTime>,

    #[serde(rename = "om_shipped_datetime")]
    #[serde(default)]
    #[serde(deserialize_with = "empty_date_time_as_option")]
    pub shipped_datetime: Option<NaiveDateTime>,

    #[serde(rename = "om_delivered_datetime")]
    #[serde(default)]
    #[serde(deserialize_with = "empty_date_time_as_option")]
    pub delivered_datetime: Option<NaiveDateTime>,

    #[serde(rename = "om_verified_datetime")]
    #[serde(default)]
    #[serde(deserialize_with = "empty_date_time_as_option")]
    pub verified_datetime: Option<NaiveDateTime>,

    #[serde(default)]
    pub om_status: Option<InvoiceStatus>,
    #[serde(default)]
    pub om_type: Option<InvoiceType>,

    #[serde(deserialize_with = "empty_str_as_option")]
    #[serde(default)]
    pub om_colour: Option<String>,
}

pub struct InvoiceTranslation {}
impl RemotePullTranslation for InvoiceTranslation {
    fn try_translate_pull(
        &self,
        connection: &StorageConnection,
        sync_record: &RemoteSyncBufferRow,
    ) -> Result<Option<IntegrationRecord>, anyhow::Error> {
        let table_name = TRANSLATION_RECORD_TRANSACT;
        if sync_record.table_name != table_name {
            return Ok(None);
        }

        let data = serde_json::from_str::<LegacyTransactRow>(&sync_record.data)?;

        let name = NameRepository::new(connection)
            .find_one_by_id(&data.name_ID)
            .ok()
            .flatten()
            .ok_or(anyhow::Error::msg(format!(
                "Missing name: {}",
                data.name_ID
            )))?;

        let name_store_id = StoreRowRepository::new(connection)
            .find_one_by_name_id(&data.name_ID)?
            .map(|store_row| store_row.id);

        let invoice_type = invoice_type(&data._type, &name).ok_or(anyhow::Error::msg(format!(
            "Unsupported invoice type: {:?}",
            data._type
        )))?;
        let invoice_status = invoice_status(&invoice_type, &data).ok_or(anyhow::Error::msg(
            format!("Unsupported invoice type: {:?}", data._type),
        ))?;
        let mapping = map_legacy(&invoice_type, &data);

        Ok(Some(IntegrationRecord::from_upsert(
            IntegrationUpsertRecord::Invoice(InvoiceRow {
                id: data.ID,
                user_id: data.user_id,
                store_id: data.store_ID,
                name_id: data.name_ID,
                name_store_id,
                invoice_number: data.invoice_num,
                r#type: data.om_type.map(|t| t.to_domain()).unwrap_or(invoice_type),
                status: data
                    .om_status
                    .map(|s| s.to_domain())
                    .unwrap_or(invoice_status),
                on_hold: data.hold,
                comment: data.comment,
                their_reference: data.their_ref,

                // new om field mappings
                created_datetime: mapping.created_datetime,
                allocated_datetime: mapping.allocated_datetime,
                picked_datetime: mapping.picked_datetime,
                shipped_datetime: mapping.shipped_datetime,
                delivered_datetime: mapping.delivered_datetime,
                verified_datetime: mapping.verified_datetime,
                colour: mapping.colour,

                requisition_id: data.requisition_ID,
                linked_invoice_id: data.linked_transaction_id,
                transport_reference: data.transport_reference,
            }),
        )))
    }
}

fn invoice_type(_type: &LegacyTransactType, name: &NameRow) -> Option<InvoiceRowType> {
    if name.code == INVENTORY_ADJUSTMENT_NAME_CODE {
        return Some(InvoiceRowType::InventoryAdjustment);
    }
    match _type {
        LegacyTransactType::Si => Some(InvoiceRowType::InboundShipment),
        LegacyTransactType::Ci => Some(InvoiceRowType::OutboundShipment),
        _ => return None,
    }
}

/// Helper struct for new om_* fields mappings
struct LegacyMapping {
    created_datetime: NaiveDateTime,
    picked_datetime: Option<NaiveDateTime>,
    delivered_datetime: Option<NaiveDateTime>,
    allocated_datetime: Option<NaiveDateTime>,
    shipped_datetime: Option<NaiveDateTime>,
    verified_datetime: Option<NaiveDateTime>,
    colour: Option<String>,
}
/// Either make use of om_* fields, if present, or do a best afford mapping
fn map_legacy(invoice_type: &InvoiceRowType, data: &LegacyTransactRow) -> LegacyMapping {
    if let Some(created_datetime) = data.created_datetime {
        return LegacyMapping {
            created_datetime,
            picked_datetime: data.picked_datetime,
            delivered_datetime: data.delivered_datetime,
            allocated_datetime: data.allocated_datetime,
            shipped_datetime: data.shipped_datetime,
            verified_datetime: data.verified_datetime,
            colour: data.om_colour.clone(),
        };
    }

    let mut mapping = LegacyMapping {
        created_datetime: NaiveDateTime::new(data.entry_date, data.entry_time),
        picked_datetime: None,
        delivered_datetime: None,
        allocated_datetime: None,
        shipped_datetime: data
            .ship_date
            .map(|ship_date| date_and_time_to_datatime(ship_date, 0)),
        verified_datetime: None,
        colour: Some(format!("#{:06X}", data.Colour)),
    };

    let confirm_datetime = data
        .confirm_date
        .map(|confirm_date| NaiveDateTime::new(confirm_date, data.confirm_time));

    match invoice_type {
        InvoiceRowType::OutboundShipment => {
            mapping.picked_datetime = confirm_datetime;
        }
        InvoiceRowType::InboundShipment => {
            mapping.delivered_datetime = confirm_datetime;
        }
        InvoiceRowType::InventoryAdjustment => {
            mapping.delivered_datetime = confirm_datetime;
        }
    };
    mapping
}

fn to_legacy_confirm_time(
    invoice_type: &InvoiceRowType,
    picked_datetime: Option<NaiveDateTime>,
    delivered_datetime: Option<NaiveDateTime>,
) -> (Option<NaiveDate>, NaiveTime) {
    let datetime = match invoice_type {
        InvoiceRowType::OutboundShipment => picked_datetime,
        InvoiceRowType::InboundShipment => delivered_datetime,
        InvoiceRowType::InventoryAdjustment => delivered_datetime,
    };

    let date = datetime.map(|datetime| datetime.date());
    let time = datetime
        .map(|datetime| datetime.time())
        .unwrap_or(NaiveTime::from_hms(0, 0, 0));
    (date, time)
}

fn invoice_status(
    invoice_type: &InvoiceRowType,
    data: &LegacyTransactRow,
) -> Option<InvoiceRowStatus> {
    let status = match invoice_type {
        // outbound
        InvoiceRowType::OutboundShipment => match data.status {
            LegacyTransactStatus::Nw => InvoiceRowStatus::New,
            LegacyTransactStatus::Sg => InvoiceRowStatus::New,
            LegacyTransactStatus::Cn => InvoiceRowStatus::Picked,
            LegacyTransactStatus::Fn => InvoiceRowStatus::Shipped,
            LegacyTransactStatus::Wp => return None,
            LegacyTransactStatus::Wf => return None,
        },
        // inbound
        InvoiceRowType::InboundShipment => match data.status {
            LegacyTransactStatus::Sg => InvoiceRowStatus::New,
            LegacyTransactStatus::Nw => InvoiceRowStatus::New,
            LegacyTransactStatus::Cn => InvoiceRowStatus::Delivered,
            LegacyTransactStatus::Fn => InvoiceRowStatus::Verified,
            LegacyTransactStatus::Wp => return None,
            LegacyTransactStatus::Wf => return None,
        },

        InvoiceRowType::InventoryAdjustment => match data.status {
            LegacyTransactStatus::Nw => InvoiceRowStatus::New,
            LegacyTransactStatus::Sg => InvoiceRowStatus::New,
            LegacyTransactStatus::Cn => InvoiceRowStatus::Verified,
            LegacyTransactStatus::Fn => InvoiceRowStatus::Verified,
            LegacyTransactStatus::Wp => return None,
            LegacyTransactStatus::Wf => return None,
        },
    };
    Some(status)
}

impl RemotePushUpsertTranslation for InvoiceTranslation {
    fn try_translate_push(
        &self,
        connection: &StorageConnection,
        changelog: &ChangelogRow,
    ) -> Result<Option<Vec<PushUpsertRecord>>, anyhow::Error> {
        if changelog.table_name != ChangelogTableName::Invoice {
            return Ok(None);
        }
        let table_name = TRANSLATION_RECORD_TRANSACT;

        let InvoiceRow {
            id,
            user_id,
            name_id,
            name_store_id: _,
            store_id,
            invoice_number,
            r#type,
            status,
            on_hold,
            comment,
            their_reference,
            created_datetime,
            allocated_datetime,
            picked_datetime,
            shipped_datetime,
            delivered_datetime,
            verified_datetime,
            colour,
            requisition_id,
            linked_invoice_id,
            transport_reference,
        } = InvoiceRepository::new(connection).find_one_by_id(&changelog.row_id)?;

        let _type = legacy_invoice_type(&r#type).ok_or(anyhow::Error::msg(format!(
            "Invalid invoice type: {:?}",
            r#type
        )))?;
        let legacy_status = legacy_invoice_status(&r#type, &status).ok_or(anyhow::Error::msg(
            format!("Invalid invoice status: {:?}", r#status),
        ))?;
        let confirm_datetime = to_legacy_confirm_time(&r#type, picked_datetime, delivered_datetime);
        let legacy_row = LegacyTransactRow {
            ID: id.clone(),
            user_id,
            name_ID: name_id,
            store_ID: store_id.clone(),
            invoice_num: invoice_number,
            _type,
            status: legacy_status,
            hold: on_hold,
            comment,
            their_ref: their_reference,
            Colour: colour
                .as_ref()
                .map(|colour| parse_html_colour(colour))
                .unwrap_or(0),
            requisition_ID: requisition_id,
            linked_transaction_id: linked_invoice_id,
            entry_date: created_datetime.date(),
            entry_time: created_datetime.time(),
            ship_date: shipped_datetime
                .map(|shipped_datetime| date_from_date_time(&shipped_datetime)),
            arrival_date_actual: delivered_datetime
                .map(|delivered_datetime| date_from_date_time(&delivered_datetime)),
            confirm_date: confirm_datetime.0,
            confirm_time: confirm_datetime.1,

            mode: TransactMode::Store,
            transport_reference,
            created_datetime: Some(created_datetime),
            allocated_datetime,
            picked_datetime,
            shipped_datetime,
            delivered_datetime,
            verified_datetime,
            om_status: Some(InvoiceStatus::from_domain(status)),
            om_type: Some(InvoiceType::from_domain(r#type)),
            om_colour: colour,
        };

        Ok(Some(vec![PushUpsertRecord {
            sync_id: changelog.id,
            store_id: Some(store_id),
            table_name,
            record_id: id,
            data: serde_json::to_value(&legacy_row)?,
        }]))
    }
}

fn parse_html_colour(colour: &str) -> i32 {
    i32::from_str_radix(&colour[1..], 16).unwrap_or(0)
}

fn legacy_invoice_type(_type: &InvoiceRowType) -> Option<LegacyTransactType> {
    let t = match _type {
        InvoiceRowType::OutboundShipment => LegacyTransactType::Ci,
        InvoiceRowType::InboundShipment => LegacyTransactType::Si,
        // Always use supplier invoice. omSupply can contain incoming and outgoing lines so there is
        // no clear mapping to Ci or Si here.
        InvoiceRowType::InventoryAdjustment => LegacyTransactType::Si,
    };
    return Some(t);
}

fn legacy_invoice_status(
    t: &InvoiceRowType,
    status: &InvoiceRowStatus,
) -> Option<LegacyTransactStatus> {
    let status = match t {
        InvoiceRowType::OutboundShipment => match status {
            InvoiceRowStatus::New => LegacyTransactStatus::Sg,
            InvoiceRowStatus::Allocated => LegacyTransactStatus::Sg,
            InvoiceRowStatus::Picked => LegacyTransactStatus::Cn,
            InvoiceRowStatus::Shipped => LegacyTransactStatus::Fn,
            InvoiceRowStatus::Delivered => LegacyTransactStatus::Fn,
            InvoiceRowStatus::Verified => LegacyTransactStatus::Fn,
        },
        InvoiceRowType::InboundShipment => match status {
            InvoiceRowStatus::New => LegacyTransactStatus::Nw,
            InvoiceRowStatus::Allocated => LegacyTransactStatus::Nw,
            InvoiceRowStatus::Picked => LegacyTransactStatus::Nw,
            InvoiceRowStatus::Shipped => LegacyTransactStatus::Nw,
            InvoiceRowStatus::Delivered => LegacyTransactStatus::Cn,
            InvoiceRowStatus::Verified => LegacyTransactStatus::Fn,
        },
        InvoiceRowType::InventoryAdjustment => match status {
            InvoiceRowStatus::New => LegacyTransactStatus::Nw,
            InvoiceRowStatus::Allocated => LegacyTransactStatus::Nw,
            InvoiceRowStatus::Picked => LegacyTransactStatus::Nw,
            InvoiceRowStatus::Shipped => LegacyTransactStatus::Nw,
            InvoiceRowStatus::Delivered => LegacyTransactStatus::Nw,
            InvoiceRowStatus::Verified => LegacyTransactStatus::Fn,
        },
    };
    Some(status)
}

#[cfg(test)]
mod tests {
    use repository::{mock::MockDataInserts, test_db::setup_all};

    use crate::sync::translation_remote::{
        invoice::InvoiceTranslation, pull::RemotePullTranslation,
        test_data::transact::get_test_transact_records,
    };

    #[actix_rt::test]
    async fn test_invoice_translation() {
        let (_, connection, _, _) =
            setup_all("test_invoice_translation", MockDataInserts::all()).await;

        let translator = InvoiceTranslation {};
        for record in get_test_transact_records() {
            let translation_result = translator
                .try_translate_pull(&connection, &record.remote_sync_buffer_row)
                .unwrap();

            assert_eq!(translation_result, record.translated_record);
        }
    }
}
