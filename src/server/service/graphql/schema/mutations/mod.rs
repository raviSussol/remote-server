use crate::business::insert_supplier_invoice;
use crate::database::repository::{
    InvoiceRepository, RepositoryError, RequisitionLineRepository, RequisitionRepository,
};
use crate::database::schema::{RequisitionLineRow, RequisitionRow};
use crate::server::service::graphql::schema::types::{
    InputRequisitionLine, Requisition, RequisitionType,
};
use crate::server::service::graphql::ContextExt;

use async_graphql::*;

use self::supplier_invoice::{InsertSupplierInvoiceInput, InvoiceOrInsertSupplierInvoiceError};

pub mod supplier_invoice;

pub struct Mutations;

#[Object]
impl Mutations {
    async fn insert_supplier_invoice(
        &self,
        ctx: &Context<'_>,
        input: InsertSupplierInvoiceInput,
    ) -> InvoiceOrInsertSupplierInvoiceError {
        let invoice_repository = ctx.get_repository::<InvoiceRepository>();
        let new_id = input.id.clone();
        let insert_result = insert_supplier_invoice(ctx, input).await;

        InvoiceOrInsertSupplierInvoiceError::new(new_id, insert_result, invoice_repository).await
    }

    async fn insert_requisition(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the requisition")] id: String,
        #[graphql(desc = "id of the receiving store")] name_id: String,
        #[graphql(desc = "id of the sending store")] store_id: String,
        #[graphql(desc = "type of the requisition")] type_of: RequisitionType,
        #[graphql(desc = "requisition lines attached to the requisition")] requisition_lines: Vec<
            InputRequisitionLine,
        >,
    ) -> Requisition {
        let requisition_row = RequisitionRow {
            id: id.clone(),
            name_id,
            store_id,
            type_of: type_of.into(),
        };

        let requisition_repository = ctx.get_repository::<RequisitionRepository>();
        let requisition_line_repository = ctx.get_repository::<RequisitionLineRepository>();

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

// Common Mutation Errors

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
pub enum ForeignKeys {
    OtherPartyId,
}

#[derive(SimpleObject)]
pub struct ForeignKeyError {
    pub key: ForeignKeys,
    pub key_id: String,
    pub description: String,
}

#[derive(SimpleObject)]
pub struct RecordExists {
    pub description: String,
}

pub struct DBError(pub RepositoryError);

#[Object]
impl DBError {
    pub async fn description(&self) -> &'static str {
        "Dabase Error"
    }

    pub async fn full_error(&self) -> String {
        format!("{:#}", self.0)
    }
}
