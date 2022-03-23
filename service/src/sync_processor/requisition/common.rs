use crate::{
    number::next_number,
    requisition::common::get_lines_for_requisition,
    sync_processor::{ProcessRecordError, RecordForProcessing},
};
use chrono::Utc;
use repository::EqualFilter;
use repository::{
    schema::{
        NumberRowType, RequisitionLineRow, RequisitionRow, RequisitionRowStatus, RequisitionRowType,
    },
    ItemStats, ItemStatsFilter, ItemStatsRepository, RequisitionLineRowRepository,
    RequisitionRowRepository, StorageConnection,
};
use util::{inline_init, uuid::uuid};

pub fn can_create_response_requisition(
    source_requisition: &RequisitionRow,
    record_for_processing: &RecordForProcessing,
) -> bool {
    if !record_for_processing.is_other_party_active_on_site {
        return false;
    }

    if record_for_processing.linked_record.is_some() {
        return false;
    }

    if source_requisition.r#type != RequisitionRowType::Request {
        return false;
    }

    if source_requisition.status != RequisitionRowStatus::Sent {
        return false;
    }

    true
}

pub fn generate_and_integrate_linked_requisition(
    connection: &StorageConnection,
    source_requisition: &RequisitionRow,
    record_for_processing: &RecordForProcessing,
) -> Result<(RequisitionRow, Vec<RequisitionLineRow>), ProcessRecordError> {
    let requisition_row =
        generate_linked_requisition(connection, &source_requisition, record_for_processing)?;
    let requisition_line_rows =
        generate_linked_requisition_lines(connection, &requisition_row, &source_requisition)?;

    RequisitionRowRepository::new(connection).upsert_one(&requisition_row)?;

    let requisition_line_row_repository = RequisitionLineRowRepository::new(connection);

    for line in requisition_line_rows.iter() {
        requisition_line_row_repository.upsert_one(line)?;
    }

    Ok((requisition_row, requisition_line_rows))
}

pub fn generate_linked_requisition(
    connection: &StorageConnection,
    source_requisition: &RequisitionRow,
    record_for_processing: &RecordForProcessing,
) -> Result<RequisitionRow, ProcessRecordError> {
    let store_id = record_for_processing
        .other_party_store
        .clone()
        .ok_or(ProcessRecordError::OtherPartyStoreIsNotFound(
            record_for_processing.clone(),
        ))?
        .id;

    let name_id = record_for_processing.source_name.id.clone();

    let result = RequisitionRow {
        id: uuid(),
        requisition_number: next_number(
            connection,
            &NumberRowType::ResponseRequisition,
            &store_id,
        )?,
        name_id,
        store_id,
        r#type: RequisitionRowType::Response,
        status: RequisitionRowStatus::New,
        created_datetime: Utc::now().naive_utc(),
        their_reference: source_requisition.their_reference.clone(),
        max_months_of_stock: source_requisition.max_months_of_stock.clone(),
        min_months_of_stock: source_requisition.min_months_of_stock.clone(),
        linked_requisition_id: Some(source_requisition.id.clone()),
        expected_delivery_date: source_requisition.expected_delivery_date,
        // Default
        user_id: None,
        sent_datetime: None,
        finalised_datetime: None,
        colour: None,
        comment: None,
    };

    Ok(result)
}

fn generate_linked_requisition_lines(
    connection: &StorageConnection,
    linked_requisition: &RequisitionRow,
    source_requisition: &RequisitionRow,
) -> Result<Vec<RequisitionLineRow>, ProcessRecordError> {
    let source_lines = get_lines_for_requisition(connection, &source_requisition.id)?;

    let mut new_lines = Vec::new();

    for source_line in source_lines.into_iter() {
        let item_id = source_line.requisition_line_row.item_id.clone();
        let item_stats = get_item_stats(connection, &linked_requisition.store_id, &item_id)?;

        let new_row = inline_init(|r: &mut RequisitionLineRow| {
            r.id = uuid();
            r.requisition_id = linked_requisition.id.clone();
            r.item_id = source_line.requisition_line_row.item_id;
            r.requested_quantity = source_line.requisition_line_row.requested_quantity;
            r.suggested_quantity = source_line.requisition_line_row.suggested_quantity;
            r.available_stock_on_hand = item_stats.available_stock_on_hand();
            r.average_monthly_consumption = item_stats.average_monthly_consumption();
        });

        new_lines.push(new_row);
    }

    Ok(new_lines)
}

fn get_item_stats(
    connection: &StorageConnection,
    store_id: &str,
    item_id: &str,
) -> Result<ItemStats, ProcessRecordError> {
    let repository = ItemStatsRepository::new(&connection);

    let filter = ItemStatsFilter::new().item_id(EqualFilter::equal_any(vec![item_id.to_string()]));

    let result = repository.query_one(store_id, None, filter)?.ok_or(
        ProcessRecordError::CannotFindStatsForItemAndStore {
            store_id: store_id.to_string(),
            item_id: item_id.to_string(),
        },
    )?;

    Ok(result)
}
