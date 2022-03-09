use log::{info, warn};
use repository::{
    schema::{ChangelogAction, ChangelogRow},
    StorageConnection,
};

use crate::sync::{
    translation_remote::{
        invoice::InvoiceTranslation, invoice_line::InvoiceLineTranslation,
        name_store_join::NameStoreJoinTranslation, number::NumberTranslation,
        requisition::RequisitionTranslation, requisition_line::RequisitionLineTranslation,
        stock_line::StockLineTranslation, stocktake::StocktakeTranslation,
        stocktake_line::StocktakeLineTranslation, table_name_to_central,
    },
    SyncTranslationError,
};

// Translates push upserts
pub trait RemotePushUpsertTranslation {
    fn try_translate_push(
        &self,
        connection: &StorageConnection,
        changelog: &ChangelogRow,
    ) -> Result<Option<Vec<PushUpsertRecord>>, SyncTranslationError>;
}

#[derive(Debug)]
pub struct PushUpsertRecord {
    pub sync_id: i64,
    pub store_id: Option<String>,
    /// The translated table name
    pub table_name: &'static str,
    pub record_id: String,
    pub data: serde_json::Value,
}

pub struct PushDeleteRecord {
    pub sync_id: i64,
    /// The translated table name
    pub table_name: &'static str,
    pub record_id: String,
}

pub enum PushRecord {
    Upsert(PushUpsertRecord),
    Delete(PushDeleteRecord),
}

pub fn to_push_translation_error(
    table_name: &'static str,
    err: anyhow::Error,
    changelog: &ChangelogRow,
) -> SyncTranslationError {
    SyncTranslationError {
        table_name,
        source: err,
        record: format!("{:?}", changelog),
    }
}

pub fn translate_changelog(
    connection: &StorageConnection,
    changelog: &ChangelogRow,
    results: &mut Vec<PushRecord>,
) -> Result<(), SyncTranslationError> {
    match changelog.row_action {
        ChangelogAction::Upsert => {
            let translations: Vec<Box<dyn RemotePushUpsertTranslation>> = vec![
                Box::new(NumberTranslation {}),
                Box::new(StockLineTranslation {}),
                // Don't push name store joins for now
                Box::new(NameStoreJoinTranslation {}),
                Box::new(InvoiceTranslation {}),
                Box::new(InvoiceLineTranslation {}),
                Box::new(StocktakeTranslation {}),
                Box::new(StocktakeLineTranslation {}),
                Box::new(RequisitionTranslation {}),
                Box::new(RequisitionLineTranslation {}),
            ];
            for translation in translations {
                if let Some(records) = translation.try_translate_push(connection, changelog)? {
                    info!("Push record upserts: {:?}", records);
                    for record in records {
                        results.push(PushRecord::Upsert(record));
                    }
                    return Ok(());
                }
            }
        }
        ChangelogAction::Delete => {
            info!(
                "Push record deletion: table: \"{:?}\", record id: {}",
                changelog.table_name, changelog.row_id
            );
            results.push(PushRecord::Delete(PushDeleteRecord {
                sync_id: changelog.id,
                table_name: table_name_to_central(&changelog.table_name),
                record_id: changelog.row_id.clone(),
            }));
            return Ok(());
        }
    };

    warn!("Unhandled push changlog: {:?}", changelog);
    Ok(())
}
