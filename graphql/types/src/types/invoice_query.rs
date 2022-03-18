use super::{InvoiceLineConnector, NameNode, RequisitionNode, StoreNode, UserNode};
use async_graphql::*;
use chrono::{DateTime, Utc};
use dataloader::DataLoader;

use graphql_core::loader::{
    InvoiceByIdLoader, InvoiceLineByInvoiceIdLoader, NameByIdLoaderInput, UserAccountLoader,
};
use graphql_core::{
    loader::{InvoiceStatsLoader, NameByIdLoader, RequisitionsByIdLoader, StoreByIdLoader},
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};
use repository::schema::{InvoiceRow, InvoiceRowStatus, InvoiceRowType, PricingRow};

use repository::{unknown_user, Invoice};
use serde::Serialize;
use service::{usize_to_u32, ListResult};

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceNodeType {
    OutboundShipment,
    InboundShipment,
    InventoryAdjustment,
}

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] // only needed to be comparable in tests
pub enum InvoiceNodeStatus {
    /// Outbound Shipment: available_number_of_packs in a stock line gets
    /// updated when items are added to the invoice.
    /// Inbound Shipment: No stock changes in this status, only manually entered
    /// inbound Shipments have new status
    New,
    /// General description: Outbound Shipment is ready for picking (all unallocated lines need to be fullfilled)
    /// Outbound Shipment: Invoice can only be turned to allocated status when
    /// all unallocated lines are fullfilled
    /// Inbound Shipment: not applicable
    Allocated,
    /// General description: Outbound Shipment was picked from shelf and ready for Shipment
    /// Outbound Shipment: available_number_of_packs and
    /// total_number_of_packs get updated when items are added to the invoice
    /// Inbound Shipment: For inter store stock transfers an inbound Shipment
    /// is created when corresponding outbound Shipment is picked and ready for
    /// Shipment, inbound Shipment is not editable in this status
    Picked,
    /// General description: Outbound Shipment is sent out for delivery
    /// Outbound Shipment: Becomes not editable
    /// Inbound Shipment: For inter store stock transfers an inbound Shipment
    /// becomes editable when this status is set as a result of corresponding
    /// outbound Shipment being chagned to shipped (this is similar to New status)
    Shipped,
    /// General description: Inbound Shipment was received
    /// Outbound Shipment: Status is updated based on corresponding inbound Shipment
    /// Inbound Shipment: Stock is introduced and can be issued
    Delivered,
    /// General description: Received inbound Shipment was counted and verified
    /// Outbound Shipment: Status is updated based on corresponding inbound Shipment
    /// Inbound Shipment: Becomes not editable
    Verified,
}

pub struct InvoiceNode {
    pub invoice: Invoice,
}

#[derive(SimpleObject)]
pub struct InvoiceConnector {
    total_count: u32,
    nodes: Vec<InvoiceNode>,
}

#[Object]
impl InvoiceNode {
    pub async fn id(&self) -> &str {
        &self.row().id
    }

    pub async fn other_party_name(&self) -> &str {
        self.invoice.other_party_name()
    }

    pub async fn other_party_id(&self) -> &str {
        self.invoice.other_party_id()
    }

    pub async fn other_party_store(&self, ctx: &Context<'_>) -> Result<Option<StoreNode>> {
        let other_party_store_id = match self.invoice.other_party_store_id() {
            Some(other_party_store_id) => other_party_store_id,
            None => return Ok(None),
        };

        let loader = ctx.get_loader::<DataLoader<StoreByIdLoader>>();
        Ok(loader
            .load_one(other_party_store_id.clone())
            .await?
            .map(StoreNode::from_domain))
    }

    /// User that last edited invoice, if user is not found in system default unknow user is returned
    /// Null is returned for transfers, where inbound has not been edited yet
    /// Null is also returned for system created invoices like inventory adjustments
    pub async fn user(&self, ctx: &Context<'_>) -> Result<Option<UserNode>> {
        let loader = ctx.get_loader::<DataLoader<UserAccountLoader>>();

        let user_id = match &self.row().user_id {
            Some(user_id) => user_id,
            None => return Ok(None),
        };

        let result = loader
            .load_one(user_id.clone())
            .await?
            .unwrap_or(unknown_user());

        Ok(Some(UserNode::from_domain(result)))
    }

    pub async fn r#type(&self) -> InvoiceNodeType {
        InvoiceNodeType::from_domain(&self.row().r#type)
    }

    pub async fn status(&self) -> InvoiceNodeStatus {
        InvoiceNodeStatus::from_domain(&self.row().status)
    }

    pub async fn invoice_number(&self) -> i64 {
        self.row().invoice_number
    }

    pub async fn their_reference(&self) -> &Option<String> {
        &self.row().their_reference
    }

    pub async fn comment(&self) -> &Option<String> {
        &self.row().comment
    }

    pub async fn on_hold(&self) -> bool {
        self.row().on_hold
    }

    pub async fn created_datetime(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(self.row().created_datetime, Utc)
    }

    pub async fn allocated_datetime(&self) -> Option<DateTime<Utc>> {
        self.row()
            .allocated_datetime
            .map(|v| DateTime::<Utc>::from_utc(v, Utc))
    }

    pub async fn picked_datetime(&self) -> Option<DateTime<Utc>> {
        self.row()
            .picked_datetime
            .map(|v| DateTime::<Utc>::from_utc(v, Utc))
    }

    pub async fn shipped_datetime(&self) -> Option<DateTime<Utc>> {
        self.row()
            .shipped_datetime
            .map(|v| DateTime::<Utc>::from_utc(v, Utc))
    }

    pub async fn delivered_datetime(&self) -> Option<DateTime<Utc>> {
        self.row()
            .delivered_datetime
            .map(|v| DateTime::<Utc>::from_utc(v, Utc))
    }

    pub async fn verified_datetime(&self) -> Option<DateTime<Utc>> {
        self.row()
            .verified_datetime
            .map(|v| DateTime::<Utc>::from_utc(v, Utc))
    }

    pub async fn colour(&self) -> &Option<String> {
        &self.row().colour
    }

    /// Response Requisition that is the origin of this Outbound Shipment
    /// Or Request Requisition for Inbound Shipment that Originated from Outbound Shipment (linked through Response Requisition)
    pub async fn requisition(&self, ctx: &Context<'_>) -> Result<Option<RequisitionNode>> {
        let requisition_id = if let Some(id) = &self.row().requisition_id {
            id
        } else {
            return Ok(None);
        };

        let loader = ctx.get_loader::<DataLoader<RequisitionsByIdLoader>>();

        Ok(loader
            .load_one(requisition_id.clone())
            .await?
            .map(RequisitionNode::from_domain))
    }

    /// Inbound Shipment <-> Outbound Shipment, where Inbound Shipment originated from Outbound Shipment
    pub async fn linked_shipment(&self, ctx: &Context<'_>) -> Result<Option<InvoiceNode>> {
        let linked_invoice_id = if let Some(id) = &self.row().linked_invoice_id {
            id
        } else {
            return Ok(None);
        };

        let loader = ctx.get_loader::<DataLoader<InvoiceByIdLoader>>();
        Ok(loader
            .load_one(linked_invoice_id.to_string())
            .await?
            .map(InvoiceNode::from_domain))
    }

    pub async fn lines(&self, ctx: &Context<'_>) -> Result<InvoiceLineConnector> {
        let loader = ctx.get_loader::<DataLoader<InvoiceLineByInvoiceIdLoader>>();
        let result_option = loader.load_one(self.row().id.to_string()).await?;

        Ok(InvoiceLineConnector::from_vec(
            result_option.unwrap_or(vec![]),
        ))
    }

    pub async fn pricing(&self, ctx: &Context<'_>) -> Result<PricingNode> {
        let loader = ctx.get_loader::<DataLoader<InvoiceStatsLoader>>();
        let default = PricingRow {
            invoice_id: self.row().id.clone(),
            total_before_tax: 0.0,
            total_after_tax: 0.0,
            stock_total_before_tax: 0.0,
            stock_total_after_tax: 0.0,
            service_total_before_tax: 0.0,
            service_total_after_tax: 0.0,
            tax_percentage: None,
        };

        let result_option = loader.load_one(self.row().id.to_string()).await?;

        Ok(PricingNode {
            invoice_pricing: result_option.unwrap_or(default),
        })
    }

    pub async fn other_party(&self, ctx: &Context<'_>, store_id: String) -> Result<NameNode> {
        let loader = ctx.get_loader::<DataLoader<NameByIdLoader>>();

        let response_option = loader
            .load_one(NameByIdLoaderInput::new(&store_id, &self.row().name_id))
            .await?;

        response_option.map(NameNode::from_domain).ok_or(
            StandardGraphqlError::InternalError(format!(
                "Cannot find name ({}) linked to invoice ({})",
                &self.row().name_id,
                &self.row().id
            ))
            .extend(),
        )
    }
}

impl InvoiceNode {
    pub fn from_domain(invoice: Invoice) -> InvoiceNode {
        InvoiceNode { invoice }
    }
    pub fn row(&self) -> &InvoiceRow {
        &self.invoice.invoice_row
    }
}

// INVOICE LINE PRICING
pub struct PricingNode {
    pub invoice_pricing: PricingRow,
}

#[Object]
impl PricingNode {
    // total

    pub async fn total_before_tax(&self) -> f64 {
        self.invoice_pricing.total_before_tax
    }

    pub async fn total_after_tax(&self) -> f64 {
        self.invoice_pricing.total_after_tax
    }

    // stock

    pub async fn stock_total_before_tax(&self) -> f64 {
        self.invoice_pricing.stock_total_before_tax
    }

    pub async fn stock_total_after_tax(&self) -> f64 {
        self.invoice_pricing.stock_total_after_tax
    }

    // service

    pub async fn service_total_before_tax(&self) -> f64 {
        self.invoice_pricing.service_total_before_tax
    }

    pub async fn service_total_after_tax(&self) -> f64 {
        self.invoice_pricing.service_total_after_tax
    }

    // tax

    pub async fn tax_percentage(&self) -> &Option<f64> {
        &self.invoice_pricing.tax_percentage
    }
}

impl InvoiceConnector {
    pub fn from_domain(invoices: ListResult<Invoice>) -> InvoiceConnector {
        InvoiceConnector {
            total_count: invoices.count,
            nodes: invoices
                .rows
                .into_iter()
                .map(InvoiceNode::from_domain)
                .collect(),
        }
    }

    pub fn from_vec(invoices: Vec<Invoice>) -> InvoiceConnector {
        InvoiceConnector {
            total_count: usize_to_u32(invoices.len()),
            nodes: invoices.into_iter().map(InvoiceNode::from_domain).collect(),
        }
    }
}

impl InvoiceNodeType {
    pub fn to_domain(self) -> InvoiceRowType {
        use InvoiceNodeType::*;
        match self {
            OutboundShipment => InvoiceRowType::OutboundShipment,
            InboundShipment => InvoiceRowType::InboundShipment,
            InventoryAdjustment => InvoiceRowType::InventoryAdjustment,
        }
    }

    pub fn from_domain(r#type: &InvoiceRowType) -> InvoiceNodeType {
        use InvoiceRowType::*;
        match r#type {
            OutboundShipment => InvoiceNodeType::OutboundShipment,
            InboundShipment => InvoiceNodeType::InboundShipment,
            InventoryAdjustment => InvoiceNodeType::InventoryAdjustment,
        }
    }
}

impl InvoiceNodeStatus {
    pub fn to_domain(self) -> InvoiceRowStatus {
        use InvoiceNodeStatus::*;
        match self {
            New => InvoiceRowStatus::New,
            Allocated => InvoiceRowStatus::Allocated,
            Picked => InvoiceRowStatus::Picked,
            Shipped => InvoiceRowStatus::Shipped,
            Delivered => InvoiceRowStatus::Delivered,
            Verified => InvoiceRowStatus::Verified,
        }
    }

    pub fn from_domain(status: &InvoiceRowStatus) -> InvoiceNodeStatus {
        use InvoiceRowStatus::*;
        match status {
            New => InvoiceNodeStatus::New,
            Allocated => InvoiceNodeStatus::Allocated,
            Picked => InvoiceNodeStatus::Picked,
            Shipped => InvoiceNodeStatus::Shipped,
            Delivered => InvoiceNodeStatus::Delivered,
            Verified => InvoiceNodeStatus::Verified,
        }
    }
}

#[cfg(test)]
mod test {

    use async_graphql::{EmptyMutation, Object};

    use graphql_core::{assert_graphql_query, test_helpers::setup_graphl_test_with_data};
    use repository::{
        mock::{
            mock_item_a, mock_item_b, mock_item_c, mock_name_a, mock_store_a, MockData,
            MockDataInserts,
        },
        schema::{InvoiceLineRow, InvoiceLineRowType, InvoiceRow},
        Invoice,
    };
    use serde_json::json;
    use util::inline_init;

    use crate::types::InvoiceNode;

    #[actix_rt::test]
    async fn graphq_test_invoice_pricing() {
        #[derive(Clone)]
        struct TestQuery;

        fn invoice() -> InvoiceRow {
            inline_init(|r: &mut InvoiceRow| {
                r.id = "test_invoice_pricing".to_string();
                r.name_id = mock_name_a().id;
                r.store_id = mock_store_a().id;
            })
        }
        fn line1() -> InvoiceLineRow {
            inline_init(|r: &mut InvoiceLineRow| {
                r.invoice_id = invoice().id;
                r.id = "line1_id".to_string();
                r.item_id = mock_item_a().id;
                r.total_after_tax = 110.0;
                r.total_before_tax = 100.0;
                r.tax = Some(10.0);
                r.r#type = InvoiceLineRowType::Service;
            })
        }
        fn line2() -> InvoiceLineRow {
            inline_init(|r: &mut InvoiceLineRow| {
                r.invoice_id = invoice().id;
                r.id = "line2_id".to_string();
                r.item_id = mock_item_b().id;
                r.total_after_tax = 50.0;
                r.total_before_tax = 50.0;
                r.tax = None;
                r.r#type = InvoiceLineRowType::StockIn;
            })
        }
        fn line3() -> InvoiceLineRow {
            inline_init(|r: &mut InvoiceLineRow| {
                r.invoice_id = invoice().id;
                r.id = "line3_id".to_string();
                r.item_id = mock_item_c().id;
                r.total_after_tax = 105.0;
                r.total_before_tax = 100.0;
                r.tax = Some(5.0);
                r.r#type = InvoiceLineRowType::StockOut;
            })
        }

        let (_, _, _, settings) = setup_graphl_test_with_data(
            TestQuery,
            EmptyMutation,
            "graphq_test_invoice_pricing",
            MockDataInserts::all(),
            Some(inline_init(|r: &mut MockData| {
                r.invoices = vec![invoice()];
                r.invoice_lines = vec![line1(), line2(), line3()];
            })),
        )
        .await;

        #[Object]
        impl TestQuery {
            pub async fn test_query(&self) -> InvoiceNode {
                InvoiceNode {
                    invoice: inline_init(|r: &mut Invoice| r.invoice_row = invoice()),
                }
            }
        }
        let total_before_tax = 50.0 + 100.0 + 100.0;
        let total_after_tax = 50.0 + 105.0 + 110.0;
        let tax_percentage_dec = (total_after_tax / total_before_tax) - 1.0;

        assert_eq!(
            total_before_tax * (1.0 + tax_percentage_dec),
            total_after_tax
        );
        let tax_percentage = tax_percentage_dec * 100.0;

        let expected = json!({
            "testQuery": {
                "pricing": {
                    "totalBeforeTax": total_before_tax,
                    "totalAfterTax": total_after_tax,
                    "stockTotalBeforeTax": 50.0 + 100.0,
                    "stockTotalAfterTax": 50.0 + 105.0,
                    "serviceTotalBeforeTax": 100.0,
                    "serviceTotalAfterTax": 110.0,
                    "taxPercentage": tax_percentage
                },
            }
        }
        );

        let query = r#"
        query {
            testQuery {
                pricing {
                    totalBeforeTax
                    totalAfterTax
                    stockTotalBeforeTax
                    stockTotalAfterTax
                    serviceTotalBeforeTax
                    serviceTotalAfterTax
                    taxPercentage  
                }
            }
        }
        "#;

        assert_graphql_query!(&settings, &query, &None, expected, None);
    }
}
