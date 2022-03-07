use super::IdPairWithPayload;
use actix_web::web::Data;
use async_graphql::dataloader::*;
use chrono::NaiveDateTime;
use repository::EqualFilter;
use repository::{ItemStats, ItemStatsFilter};
use service::service_provider::ServiceProvider;
use std::collections::HashMap;

pub struct ItemsStatsForItemLoader {
    pub service_provider: Data<ServiceProvider>,
}

pub type ItemStatsLoaderInputPayload = Option<NaiveDateTime>;
pub type ItemStatsLoaderInput = IdPairWithPayload<ItemStatsLoaderInputPayload>;
impl ItemStatsLoaderInput {
    pub fn new(store_id: &str, item_id: &str, payload: ItemStatsLoaderInputPayload) -> Self {
        ItemStatsLoaderInput {
            primary_id: store_id.to_string(),
            secondary_id: item_id.to_string(),
            payload,
        }
    }
}

#[async_trait::async_trait]
impl Loader<ItemStatsLoaderInput> for ItemsStatsForItemLoader {
    type Value = ItemStats;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        loader_inputs: &[ItemStatsLoaderInput],
    ) -> Result<HashMap<ItemStatsLoaderInput, Self::Value>, Self::Error> {
        let service_context = self.service_provider.context()?;

        let (store_id, look_back_datetime) = if let Some(loader_input) = loader_inputs.first() {
            (
                loader_input.primary_id.clone(),
                loader_input.payload.clone(),
            )
        } else {
            return Ok(HashMap::new());
        };

        let filter = ItemStatsFilter::new().item_id(EqualFilter::equal_any(
            IdPairWithPayload::get_all_secondary_ids(&loader_inputs),
        ));

        let item_stats = self.service_provider.item_stats_service.get_item_stats(
            &service_context,
            &store_id,
            look_back_datetime.clone(),
            Some(filter),
        )?;

        Ok(item_stats
            .into_iter()
            .map(|item_stat| {
                (
                    ItemStatsLoaderInput::new(&store_id, &item_stat.item_id, look_back_datetime),
                    item_stat,
                )
            })
            .collect())
    }
}
