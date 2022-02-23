use async_graphql::*;
use chrono::NaiveDate;
use dataloader::DataLoader;
use repository::{location_to_domain, StocktakeLine};
use service::{i32_to_u32, usize_to_u32};

use crate::{
    loader::{ItemLoader, StockLineByIdLoader},
    schema::types::{LocationNode, StockLineNode},
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};

use super::ItemNode;

pub struct StocktakeLineNode {
    pub line: StocktakeLine,
}

#[Object]
impl StocktakeLineNode {
    pub async fn id(&self) -> &str {
        &self.line.line.id
    }

    pub async fn stocktake_id(&self) -> &str {
        &self.line.line.stocktake_id
    }

    pub async fn stock_line(&self, ctx: &Context<'_>) -> Result<Option<StockLineNode>> {
        if let Some(ref stock_line) = self.line.stock_line {
            let loader = ctx.get_loader::<DataLoader<StockLineByIdLoader>>();
            let stock_line = loader.load_one(stock_line.id.clone()).await?.ok_or(
                StandardGraphqlError::InternalError(format!(
                    "Cannot find stock line {}",
                    stock_line.id
                ))
                .extend(),
            )?;
            Ok(Some(StockLineNode { stock_line }))
        } else {
            Ok(None)
        }
    }

    pub async fn location(&self) -> Option<LocationNode> {
        self.line.location.clone().map(|location| LocationNode {
            location: location_to_domain(location),
        })
    }

    pub async fn comment(&self) -> Option<String> {
        self.line.line.comment.clone()
    }

    pub async fn snapshot_number_of_packs(&self) -> u32 {
        i32_to_u32(self.line.line.snapshot_number_of_packs)
    }

    pub async fn counted_number_of_packs(&self) -> Option<u32> {
        self.line.line.counted_number_of_packs.map(i32_to_u32)
    }

    pub async fn item_id(&self) -> &str {
        &self.line.line.item_id
    }

    pub async fn item(&self, ctx: &Context<'_>) -> Result<ItemNode> {
        let loader = ctx.get_loader::<DataLoader<ItemLoader>>();
        let item_option = loader.load_one(self.line.line.item_id.clone()).await?;

        item_option.map(ItemNode::from_domain).ok_or(
            StandardGraphqlError::InternalError(format!(
                "Cannot find item_id {} for stocktake line id {}",
                self.line.line.item_id, self.line.line.id
            ))
            .extend(),
        )
    }

    pub async fn batch(&self) -> &Option<String> {
        &self.line.line.batch
    }

    pub async fn expiry_date(&self) -> &Option<NaiveDate> {
        &self.line.line.expiry_date
    }

    pub async fn pack_size(&self) -> Option<u32> {
        self.line.line.pack_size.map(i32_to_u32)
    }

    pub async fn cost_price_per_pack(&self) -> &Option<f64> {
        &self.line.line.cost_price_per_pack
    }

    pub async fn sell_price_per_pack(&self) -> &Option<f64> {
        &self.line.line.sell_price_per_pack
    }

    pub async fn note(&self) -> &Option<String> {
        &self.line.line.note
    }
}

#[derive(SimpleObject)]
pub struct StocktakeLineConnector {
    total_count: u32,
    nodes: Vec<StocktakeLineNode>,
}

impl StocktakeLineConnector {
    pub fn empty() -> StocktakeLineConnector {
        StocktakeLineConnector {
            total_count: 0,
            nodes: Vec::new(),
        }
    }

    pub fn from_domain_vec(from: Vec<StocktakeLine>) -> StocktakeLineConnector {
        StocktakeLineConnector {
            total_count: usize_to_u32(from.len()),
            nodes: from
                .into_iter()
                .map(|line| StocktakeLineNode { line })
                .collect(),
        }
    }
}
