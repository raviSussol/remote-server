pub mod pagination;

use crate::database::repository::{
    InvoiceLineRepository, InvoiceQueryRepository, RepositoryError, RequisitionRepository,
    StoreRepository,
};
use crate::database::schema::{InvoiceLineRow, RequisitionRow, StoreRow};
use crate::server::service::graphql::schema::types::{InvoiceLine, Requisition, Store};
use crate::server::service::graphql::ContextExt;

use self::pagination::Pagination;

use super::types::{InvoiceList, InvoiceNode, ItemList, NameList};
use async_graphql::*;

pub struct Queries;

pub struct Hammer {
    length: i32,
}

pub struct Nail {
    weight: f64,
}

pub struct HammerNode {
    hammer: Hammer,
}

pub struct NailNode {
    nail: Nail,
}

#[Object]
impl HammerNode {
    async fn length(&self) -> i32 {
        self.hammer.length
    }
}

#[Object]
impl NailNode {
    async fn weight(&self) -> f64 {
        self.nail.weight
    }
}

#[derive(SimpleObject)]
#[graphql(concrete(name = "HammerConnection", params(HammerNode)))]
#[graphql(concrete(name = "NailConnection", params(NailNode)))]
pub struct Connection<T: OutputType> {
    total_count: i32,
    nodes: Vec<T>,
}

pub struct ConnectionRequest {
    total_count: bool,
    nodes: bool,
}

impl From<&Context<'_>> for ConnectionRequest {
    fn from(ctx: &Context<'_>) -> Self {
        ConnectionRequest {
            total_count: ctx.look_ahead().field("totalCount").exists(),
            nodes: ctx.look_ahead().field("nodes").exists(),
        }
    }
}

pub struct Hammers {
    hammers: Vec<Hammer>,
    total_count: i32,
}

pub fn hammers_service(_: Option<Pagination>, request: ConnectionRequest) -> Hammers {
    let mut hammers = Vec::new();
    let mut total_count = 0;

    if request.total_count {
        // Query DB ..
        total_count = 2;
    }

    if request.nodes {
        // Query DB, with pagination
        hammers = vec![Hammer { length: 30 }, Hammer { length: 10 }]
    }

    Hammers {
        hammers,
        total_count,
    }
}

impl From<Hammer> for HammerNode {
    fn from(hammer: Hammer) -> Self {
        HammerNode { hammer }
    }
}

impl From<Hammers> for Connection<HammerNode> {
    fn from(
        Hammers {
            total_count,
            hammers,
        }: Hammers,
    ) -> Self {
        Connection {
            total_count,
            nodes: hammers.into_iter().map(HammerNode::from).collect(),
        }
    }
}

#[Object]
impl Queries {
    pub async fn hammers(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<Pagination>,
    ) -> Connection<HammerNode> {
        hammers_service(page, ctx.into()).into()
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
