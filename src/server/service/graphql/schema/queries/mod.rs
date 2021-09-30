pub mod pagination;

use crate::database::repository::{
    InvoiceLineRepository, InvoiceQueryRepository, RepositoryError, RequisitionRepository,
    StoreRepository,
};
use crate::database::schema::{InvoiceLineRow, RequisitionRow, StoreRow};
use crate::server::service::graphql::schema::types::{InvoiceLine, Requisition, Store};
use crate::server::service::graphql::ContextExt;

use super::types::{InvoiceList, InvoiceNode, ItemList, NameList, StockLineQuery};
use async_graphql::{Context, Object, SimpleObject};
use pagination::Pagination;
pub struct Queries;

#[derive(SimpleObject)]
pub struct CustomerInvoiceItem {
    item_id: String,
    item_name: String,
    lines: Vec<CustomerInvoiceItemLine>,
}
#[derive(SimpleObject)]
pub struct CustomerInvoiceItemLine {
    issue: String,
    hold: bool,
    stock_line: StockLineQuery,
}

#[Object]
impl Queries {
    pub async fn customer_invoice_item(
        &self,
        _ctx: &Context<'_>,
        item_id: String,
        invoice_id: String,
    ) -> CustomerInvoiceItem {
        todo!()
    }

    #[allow(non_snake_case)]
    pub async fn apiVersion(&self) -> String {
        "1.0".to_string()
    }

    pub async fn names(
        &self,
        _ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<Pagination>,
    ) -> NameList {
        NameList { pagination: page }
    }

    pub async fn items(
        &self,
        _ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<Pagination>,
    ) -> ItemList {
        ItemList { pagination: page }
    }

    // TODO return better error
    pub async fn invoice(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the invoice")] id: String,
    ) -> Result<InvoiceNode, RepositoryError> {
        let repository = ctx.get_repository::<InvoiceQueryRepository>();
        let invoice = repository.find_one_by_id(id.as_str()).await?;
        Ok(InvoiceNode::from(invoice))
    }

    pub async fn invoices(
        &self,
        _ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<Pagination>,
    ) -> InvoiceList {
        InvoiceList { pagination: page }
    }

    pub async fn store(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the store")] id: String,
    ) -> Store {
        let store_repository = ctx.get_repository::<StoreRepository>();

        let store_row: StoreRow = store_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get store {}", id));

        Store { store_row }
    }

    pub async fn invoice_line(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the invoice line")] id: String,
    ) -> InvoiceLine {
        let invoice_line_repository = ctx.get_repository::<InvoiceLineRepository>();

        let invoice_line_row: InvoiceLineRow = invoice_line_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get invoice line {}", id));

        InvoiceLine { invoice_line_row }
    }

    pub async fn requisition(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the requisition")] id: String,
    ) -> Requisition {
        let requisition_repository = ctx.get_repository::<RequisitionRepository>();

        let requisition_row: RequisitionRow = requisition_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get requisition {}", id));

        Requisition { requisition_row }
    }
}
