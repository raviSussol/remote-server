use crate::database::repository::{
    ItemRepository, RequisitionLineRepository, RequisitionRepository,
};
use crate::database::schema::{ItemRow, RequisitionLineRow, RequisitionRow};
use crate::server::data::Registry;
use crate::server::service::graphql::schema::types::{
    InputRequisitionLine, Item, ItemType, Requisition, RequisitionType,
};

pub struct Mutations;
#[juniper::graphql_object(context = Registry)]
impl Mutations {
    #[graphql(arguments(
        id(description = "id of the item"),
        item_name(description = "name of the item"),
        type_of(description = "type of the item"),
    ))]
    async fn insert_item(
        registry: &Registry,
        id: String,
        item_name: String,
        type_of: ItemType,
    ) -> Item {
        let item_row = ItemRow {
            id,
            item_name,
            type_of: type_of.into(),
        };

        let item_repository: &ItemRepository = &registry.item_repository;

        item_repository
            .insert_one(&item_row)
            .await
            .expect("Failed to insert item into database");

        Item { item_row }
    }

    #[graphql(arguments(
        id(description = "id of the requisition"),
        name_id(description = "id of the receiving store"),
        store_id(description = "id of the sending store"),
        type_of(description = "type of the requisition"),
        requisition_lines(description = "requisition lines attached to the requisition")
    ))]
    async fn insert_requisition(
        registry: &Registry,
        id: String,
        name_id: String,
        store_id: String,
        type_of: RequisitionType,
        requisition_lines: Vec<InputRequisitionLine>,
    ) -> Requisition {
        let requisition_row = RequisitionRow {
            id: id.clone(),
            name_id,
            store_id,
            type_of: type_of.into(),
        };

        let requisition_repository: &RequisitionRepository = &registry.requisition_repository;
        let requisition_line_repository: &RequisitionLineRepository =
            &registry.requisition_line_repository;

        requisition_repository
            .insert_one(&requisition_row)
            .await
            .expect("Failed to insert requisition into database");

        let requisition_line_rows: Vec<RequisitionLineRow> = requisition_lines
            .into_iter()
            .map(|line| RequisitionLineRow {
                id: line.id,
                requisition_id: id.clone(),
                item_id: line.item_id,
                actual_quantity: line.actual_quantity,
                suggested_quantity: line.suggested_quantity,
            })
            .collect();

        for requisition_line_row in requisition_line_rows {
            requisition_line_repository
                .insert_one(&requisition_line_row)
                .await
                .expect("Failed to insert requisition_line into database");
        }

        Requisition { requisition_row }
    }
}