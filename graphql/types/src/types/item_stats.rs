use async_graphql::*;
use service::item_stats::ItemStats;
pub struct ItemStatsNode {
    pub item_stats: ItemStats,
}

#[Object]
impl ItemStatsNode {
    pub async fn average_monthly_consumption(&self) -> f64 {
        self.item_stats.average_monthly_consumption
    }

    pub async fn available_stock_on_hand(&self) -> u32 {
        self.item_stats.available_stock_on_hand
    }

    pub async fn available_months_of_stock_on_hand(&self) -> f64 {
        if self.item_stats.average_monthly_consumption == 0.0 {
            return self.item_stats.available_stock_on_hand as f64;
        }
        self.item_stats.available_stock_on_hand as f64
            / self.item_stats.average_monthly_consumption as f64
    }
}

impl ItemStatsNode {
    pub fn from_domain(item_stats: ItemStats) -> ItemStatsNode {
        ItemStatsNode { item_stats }
    }
}
