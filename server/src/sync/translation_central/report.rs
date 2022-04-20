use crate::sync::translation_central::TRANSLATION_RECORD_REPORT;
use repository::schema::{
    report::{ReportCategory, ReportRow, ReportType},
    CentralSyncBufferRow,
};

use serde::Deserialize;

use super::{CentralPushTranslation, IntegrationUpsertRecord};

#[derive(Deserialize, Debug, PartialEq)]
pub enum LegacyReportRowType {
    #[serde(rename = "cus")]
    Cus,
    #[serde(rename = "std")]
    Std,
    #[serde(other)]
    Others,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum LegacyReportRowEditor {
    #[serde(rename = "qrep")]
    Grep,
    #[serde(rename = "ppro")]
    PPro,
    #[serde(rename = "omsupply")]
    OmSupply,
    #[serde(other)]
    Others,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum LegacyReportRowContext {
    Invoice,
    Requisition,
    Stocktake,
    Resource,
    #[serde(other)]
    LegacyContext,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct LegacyReportRow {
    #[serde(rename = "ID")]
    id: String,
    r#type: LegacyReportRowType,
    editor: LegacyReportRowEditor,
    report_name: String,
    context: LegacyReportRowContext,
}

pub struct ReportTranslation {}
impl CentralPushTranslation for ReportTranslation {
    fn try_translate(
        &self,
        sync_record: &CentralSyncBufferRow,
    ) -> Result<Option<IntegrationUpsertRecord>, anyhow::Error> {
        let table_name = TRANSLATION_RECORD_REPORT;
        if sync_record.table_name != table_name {
            return Ok(None);
        }

        let data = serde_json::from_str::<LegacyReportRow>(&sync_record.data)?;
        if data.editor != LegacyReportRowEditor::OmSupply {
            return Ok(None);
        }
        let context = match map_legacy_context(&data.context) {
            Some(context) => context,
            None => return Ok(None),
        };
        Ok(Some(IntegrationUpsertRecord::Report(ReportRow {
            id: data.id,
            name: data.report_name,
            r#type: ReportType::OmSupply,
            context,
            data: todo!(),
        })))
    }
}

fn map_legacy_context(context: &LegacyReportRowContext) -> Option<ReportCategory> {
    let category = match context {
        LegacyReportRowContext::Invoice => ReportCategory::Invoice,
        LegacyReportRowContext::Requisition => ReportCategory::Requisition,
        LegacyReportRowContext::Stocktake => ReportCategory::Stocktake,
        LegacyReportRowContext::Resource => ReportCategory::Resource,
        LegacyReportRowContext::LegacyContext => return None,
    };
    Some(category)
}

/// This sole reason for this method is to ensure there is a variant in LegacyReportRowContext for
/// each variant in ReportCategory
#[allow(dead_code)]
fn context_to_legacy(category: ReportCategory) -> LegacyReportRowContext {
    match category {
        ReportCategory::Invoice => LegacyReportRowContext::Invoice,
        ReportCategory::Requisition => LegacyReportRowContext::Requisition,
        ReportCategory::Stocktake => LegacyReportRowContext::Stocktake,
        ReportCategory::Resource => LegacyReportRowContext::Resource,
    }
}
