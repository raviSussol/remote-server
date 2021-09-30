mod item;
mod list_master;
mod list_master_line;
mod list_master_name_join;
mod name;
mod store;
pub mod test_data;

use crate::{
    database::{
        repository::{IntegrationRecord, IntegrationUpsertRecord, RepositoryError, SyncRepository},
        schema::CentralSyncBufferRow,
    },
    server::data::RepositoryRegistry,
};

use self::{
    item::LegacyItemRow, list_master::LegacyListMasterRow,
    list_master_line::LegacyListMasterLineRow, list_master_name_join::LegacyListMasterNameJoinRow,
    name::LegacyNameRow, store::LegacyStoreRow,
};

use thiserror::Error;

#[derive(Error, Debug)]
#[error("Failed to translate {table_name} sync record")]
pub struct SyncTranslationError {
    table_name: &'static str,
    source: serde_json::Error,
}

#[derive(Error, Debug)]
pub enum SyncImportError {
    #[error("Failed to translate sync records")]
    TranslationError {
        #[from]
        source: SyncTranslationError,
    },
    #[error("Failed to integrate sync records")]
    IntegrationError {
        #[from]
        source: RepositoryError,
    },
}

/// Translates sync records into the local DB schema.
/// Translated records are added to integration_records.
fn do_translation(
    sync_record: &CentralSyncBufferRow,
    integration_records: &mut IntegrationRecord,
) -> Result<(), SyncTranslationError> {
    if let Some(row) = LegacyNameRow::try_translate(sync_record)? {
        integration_records
            .upserts
            .push(IntegrationUpsertRecord::Name(row));

        return Ok(());
    }
    if let Some(row) = LegacyItemRow::try_translate(sync_record)? {
        integration_records
            .upserts
            .push(IntegrationUpsertRecord::Item(row));

        return Ok(());
    }
    if let Some(row) = LegacyStoreRow::try_translate(sync_record)? {
        // TODO: move this check up when fetching/validating/reordering the sync records?
        // ignore stores without name
        if row.name_id == "" {
            return Ok(());
        }
        integration_records
            .upserts
            .push(IntegrationUpsertRecord::Store(row));

        return Ok(());
    }

    if let Some(row) = LegacyListMasterRow::try_translate(sync_record)? {
        integration_records
            .upserts
            .push(IntegrationUpsertRecord::MasterList(row));

        return Ok(());
    }

    if let Some(row) = LegacyListMasterLineRow::try_translate(sync_record)? {
        integration_records
            .upserts
            .push(IntegrationUpsertRecord::MasterListLine(row));

        return Ok(());
    }

    if let Some(row) = LegacyListMasterNameJoinRow::try_translate(sync_record)? {
        integration_records
            .upserts
            .push(IntegrationUpsertRecord::MasterListNameJoin(row));

        return Ok(());
    }

    Ok(()) // At this point we are either ignoring records or record_types
}

pub const TRANSLATION_RECORD_NAME: &str = "name";
pub const TRANSLATION_RECORD_ITEM: &str = "item";
pub const TRANSLATION_RECORD_STORE: &str = "store";
pub const TRANSLATION_RECORD_LIST_MASTER: &str = "list_master";
pub const TRANSLATION_RECORD_LIST_MASTER_LINE: &str = "list_master_line";
pub const TRANSLATION_RECORD_LIST_MASTER_NAME_JOIN: &str = "list_master_name_join";

/// Returns a list of records that can be translated. The list is topologically sorted, i.e. items
/// at the beginning of the list don't rely on later items to be translated first.
pub const TRANSLATION_RECORDS: &[&str] = &[
    TRANSLATION_RECORD_NAME,
    TRANSLATION_RECORD_ITEM,
    TRANSLATION_RECORD_STORE,
    TRANSLATION_RECORD_LIST_MASTER,
    TRANSLATION_RECORD_LIST_MASTER_LINE,
    TRANSLATION_RECORD_LIST_MASTER_NAME_JOIN,
];

/// Imports sync records and writes them to the DB
/// If needed data records are translated to the local DB schema.
pub async fn import_sync_records(
    registry: &RepositoryRegistry,
    records: &Vec<CentralSyncBufferRow>,
) -> Result<(), SyncImportError> {
    let mut integration_records = IntegrationRecord {
        upserts: Vec::new(),
    };
    for record in records {
        do_translation(&record, &mut integration_records)?;
    }

    let sync_repo = registry.get::<SyncRepository>();
    sync_repo.integrate_records(&integration_records).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        database::repository::repository::get_repositories,
        server::data::RepositoryRegistry,
        util::{
            sync::translation::{import_sync_records, test_data::store::get_test_store_records},
            test_db,
        },
    };

    use super::test_data::{
        check_records_against_database, extract_sync_buffer_rows,
        item::{get_test_item_records, get_test_item_upsert_records},
        master_list::{get_test_master_list_records, get_test_master_list_upsert_records},
        master_list_line::get_test_master_list_line_records,
        master_list_name_join::get_test_master_list_name_join_records,
        name::{get_test_name_records, get_test_name_upsert_records},
    };

    #[actix_rt::test]
    async fn test_store_translation_insert() {
        let settings = test_db::get_test_settings("omsupply-database-translation-insert");

        test_db::setup(&settings.database).await;
        let registry = RepositoryRegistry {
            repositories: get_repositories(&settings).await,
        };

        let mut records = Vec::new();
        // Need to be in order of reference dependency
        records.append(&mut get_test_name_records());
        records.append(&mut get_test_store_records());
        records.append(&mut get_test_item_records());
        records.append(&mut get_test_master_list_records());
        records.append(&mut get_test_master_list_line_records());
        records.append(&mut get_test_master_list_name_join_records());

        import_sync_records(&registry, &extract_sync_buffer_rows(&records))
            .await
            .unwrap();

        // Asserts inside this method, to avoid repetition
        check_records_against_database(&registry, records).await;
    }

    #[actix_rt::test]
    async fn test_store_translation_upsert() {
        let settings = test_db::get_test_settings("omsupply-database-translation-upsert");

        test_db::setup(&settings.database).await;
        let registry = RepositoryRegistry {
            repositories: get_repositories(&settings).await,
        };

        let mut init_records = Vec::new();
        init_records.append(&mut get_test_name_records());
        init_records.append(&mut get_test_item_records());
        init_records.append(&mut get_test_master_list_records());
        let mut upsert_records = Vec::new();
        upsert_records.append(&mut get_test_item_upsert_records());
        upsert_records.append(&mut get_test_name_upsert_records());
        upsert_records.append(&mut get_test_master_list_upsert_records());

        let mut records = Vec::new();
        records.append(&mut init_records.iter().cloned().collect());
        records.append(&mut upsert_records.iter().cloned().collect());

        import_sync_records(&registry, &extract_sync_buffer_rows(&records))
            .await
            .unwrap();

        // Asserts inside this method, to avoid repetition
        check_records_against_database(&registry, upsert_records).await;
    }
}
