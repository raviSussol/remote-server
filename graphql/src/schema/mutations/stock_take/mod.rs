use chrono::NaiveDateTime;
use repository::{schema::StockTakeStatus, StockTake};

use async_graphql::*;

pub mod delete;
pub mod insert;
pub mod line;
pub mod update;

pub struct StockTakeNode {
    pub stock_take: StockTake,
}

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug)]
pub enum StockTakeNodeStatus {
    New,
    Finalized,
}

impl StockTakeNodeStatus {
    pub fn to_domain(&self) -> StockTakeStatus {
        match self {
            StockTakeNodeStatus::New => StockTakeStatus::New,
            StockTakeNodeStatus::Finalized => StockTakeStatus::Finalized,
        }
    }
}

fn from_domain(status: &StockTakeStatus) -> StockTakeNodeStatus {
    match status {
        StockTakeStatus::New => StockTakeNodeStatus::New,
        StockTakeStatus::Finalized => StockTakeNodeStatus::Finalized,
    }
}

#[Object]
impl StockTakeNode {
    pub async fn id(&self) -> &str {
        &self.stock_take.id
    }

    pub async fn store_id(&self) -> &str {
        &self.stock_take.store_id
    }

    pub async fn comment(&self) -> &Option<String> {
        &self.stock_take.comment
    }

    pub async fn description(&self) -> &Option<String> {
        &self.stock_take.description
    }

    pub async fn status(&self) -> StockTakeNodeStatus {
        from_domain(&self.stock_take.status)
    }

    pub async fn created_datetime(&self) -> NaiveDateTime {
        self.stock_take.created_datetime
    }

    pub async fn finalised_datetime(&self) -> Option<NaiveDateTime> {
        self.stock_take.finalised_datetime
    }

    pub async fn inventory_adjustment_id(&self) -> &Option<String> {
        &self.stock_take.inventory_adjustment_id
    }
}
