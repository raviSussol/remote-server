use crate::business::{insert_supplier_invoice, update_supplier_invoice};
use crate::database::repository::{
    InvoiceRepository, RepositoryError, RequisitionLineRepository, RequisitionRepository,
};
use crate::database::schema::{RequisitionLineRow, RequisitionRow};
use crate::server::service::graphql::schema::types::{
    InputRequisitionLine, Requisition, RequisitionType,
};
use crate::server::service::graphql::ContextExt;

use async_graphql::*;

use self::supplier_invoice::{
    InsertSupplierInvoiceInput, InvoiceOrInsertSupplierInvoiceError,
    InvoiceOrUpdateSupplierInvoiceError, UpdateSupplierInvoiceInput,
};

use super::types::{InvoiceLine, StockLineQuery};

pub mod supplier_invoice;

pub struct Mutations;

#[derive(InputObject)]
pub struct UpdateCustomerInvoiceItemInput {
    pub stock_line_id: String,
    pub issue: u32,
    pub hold: Option<bool>,
}

#[derive(SimpleObject)]
pub struct CustomerInvoiceLineUpdates {
    inserts: Vec<InvoiceLine>,
    updates: Vec<InvoiceLine>,
    deletes: Vec<String>,
}

#[derive(Union)]
pub enum UpdateCustomerInvoiceItemResult {
    Updates(CustomerInvoiceLineUpdates),
    Errors(UpdateCustomerInvoiceItemErrors),
}
#[derive(SimpleObject)]
pub struct UpdateCustomerInvoiceItemErrors {
    pub errors: Vec<UpdateCustomerInvoiceItemError>,
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum UpdateCustomerInvoiceItemError {
    DoesntBelongToCustomerInvoiceItem(DoesntBelongToCustomerInvoiceItem),
    DuplicateStockLine(DuplicateStockLine),
    ReductionBelowZero(ReductionBelowZero),
}

pub struct DoesntBelongToCustomerInvoiceItem;
#[Object]
impl DoesntBelongToCustomerInvoiceItem {
    pub async fn description(&self) -> &'static str {
        "Line does not belong to this Item"
    }

    pub async fn stock_line_id(&self) -> &'static str {
        "id"
    }
}

pub struct DuplicateStockLine;
#[Object]
impl DuplicateStockLine {
    pub async fn description(&self) -> &'static str {
        "Multiple stock lines with the same id"
    }

    pub async fn stock_line_id(&self) -> &'static str {
        "id"
    }
}

pub struct ReductionBelowZero;
#[Object]
impl ReductionBelowZero {
    pub async fn description(&self) -> &'static str {
        "Cannot reduce line below zero"
    }

    pub async fn issue(&self) -> &'static u32 {
        todo!()
    }

    pub async fn stock_line(&self) -> &'static StockLineQuery {
        todo!()
    }
}

#[Object]
impl Mutations {
    async fn update_customer_invoice_item(
        &self,
        ctx: &Context<'_>,
        input: Vec<UpdateCustomerInvoiceItemInput>,
    ) -> UpdateCustomerInvoiceItemResult {
        todo!()
    }

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

    async fn update_supplier_invoice(
        &self,
        ctx: &Context<'_>,
        input: UpdateSupplierInvoiceInput,
    ) -> InvoiceOrUpdateSupplierInvoiceError {
        let invoice_repository = ctx.get_repository::<InvoiceRepository>();
        let new_id = input.id.clone();
        let update_result = update_supplier_invoice(ctx, input).await;

        InvoiceOrUpdateSupplierInvoiceError::new(new_id, update_result, invoice_repository).await
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
    ItemId,
}

pub struct ForeignKeyError {
    pub key: ForeignKeys,
    pub id: String,
}
#[Object]
impl ForeignKeyError {
    pub async fn description(&self) -> &'static str {
        "FK record doesn't exist"
    }

    pub async fn key(&self) -> ForeignKeys {
        self.key
    }

    pub async fn id(&self) -> &str {
        &self.id
    }
}

pub struct RecordDoesNotExist;
#[Object]
impl RecordDoesNotExist {
    pub async fn description(&self) -> &'static str {
        "Record does not exist"
    }
}

pub struct RecordAlreadyExist;
#[Object]
impl RecordAlreadyExist {
    pub async fn description(&self) -> &'static str {
        "Record already exists"
    }
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

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
pub enum RangeFields {
    SellPricePerPack,
    CostPricePerPack,
    PackSize,
}

pub struct ValueOutOfRange {
    pub field: RangeFields,
    pub description: String,
}

#[Object]
impl ValueOutOfRange {
    pub async fn description(&self) -> &str {
        &self.description
    }

    pub async fn field(&self) -> &RangeFields {
        &self.field
    }
}
