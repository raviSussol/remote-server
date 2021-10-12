use crate::database::loader::StockLineByItemIdLoader;
use crate::domain::item::{Item, ItemFilter};
use crate::domain::stock_line::StockLine;
use crate::domain::{EqualFilter, SimpleStringFilter};
use crate::server::service::graphql::ContextExt;
use crate::service::{ListError, ListResult};
use async_graphql::dataloader::DataLoader;
use async_graphql::*;

use super::{
    Connector, ConnectorError, EqualFilterBoolInput, SimpleStringFilterInput, SortInput,
    StockLinesResponse,
};

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
#[graphql(remote = "crate::domain::item::ItemSortField")]
pub enum ItemSortFieldInput {
    Name,
    Code,
}
pub type ItemSortInput = SortInput<ItemSortFieldInput>;

#[derive(InputObject, Clone)]
pub struct ItemFilterInput {
    pub name: Option<SimpleStringFilterInput>,
    pub code: Option<SimpleStringFilterInput>,
    pub is_visible: Option<EqualFilterBoolInput>,
}

impl From<ItemFilterInput> for ItemFilter {
    fn from(f: ItemFilterInput) -> Self {
        ItemFilter {
            name: f.name.map(SimpleStringFilter::from),
            code: f.code.map(SimpleStringFilter::from),
            is_visible: f.is_visible.map(EqualFilter::from),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ItemNode {
    item: Item,
}

#[Object]
impl ItemNode {
    pub async fn id(&self) -> &str {
        &self.item.id
    }

    pub async fn name(&self) -> &str {
        &self.item.name
    }

    pub async fn code(&self) -> &str {
        &self.item.code
    }

    pub async fn is_visible(&self) -> bool {
        self.item.is_visible
    }

    async fn available_batches(&self, ctx: &Context<'_>) -> StockLinesResponse {
        let loader = ctx.get_loader::<DataLoader<StockLineByItemIdLoader>>();
        loader
            .load_one(self.item.id.to_string())
            .await
            .map(|result: Option<Vec<StockLine>>| result.unwrap_or(Vec::new()))
            .into()
    }
}

#[derive(Union)]
pub enum ItemsResponse {
    Error(ConnectorError),
    Response(Connector<ItemNode>),
}

impl From<Result<ListResult<Item>, ListError>> for ItemsResponse {
    fn from(result: Result<ListResult<Item>, ListError>) -> Self {
        match result {
            Ok(response) => ItemsResponse::Response(response.into()),
            Err(error) => ItemsResponse::Error(error.into()),
        }
    }
}

impl From<Item> for ItemNode {
    fn from(item: Item) -> Self {
        ItemNode { item }
    }
}
