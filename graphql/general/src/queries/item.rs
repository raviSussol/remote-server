use async_graphql::*;
use graphql_core::{
    generic_filters::{EqualFilterBoolInput, EqualFilterStringInput, SimpleStringFilterInput},
    pagination::PaginationInput,
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};
use graphql_types::types::ItemConnector;
use repository::{EqualFilter, PaginationOption, SimpleStringFilter};
use repository::{ItemFilter, ItemSort, ItemSortField};
use service::item::get_items;

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
#[graphql(remote = "repository::ItemSortField")]
#[graphql(rename_items = "camelCase")]
pub enum ItemSortFieldInput {
    Name,
    Code,
}

#[derive(InputObject)]
pub struct ItemSortInput {
    /// Sort query result by `key`
    key: ItemSortFieldInput,
    /// Sort query result is sorted descending or ascending (if not provided the default is
    /// ascending)
    desc: Option<bool>,
}

#[derive(InputObject, Clone)]
pub struct ItemFilterInput {
    pub id: Option<EqualFilterStringInput>,
    pub name: Option<SimpleStringFilterInput>,
    pub code: Option<SimpleStringFilterInput>,
    pub is_visible: Option<EqualFilterBoolInput>,
}

impl From<ItemFilterInput> for ItemFilter {
    fn from(f: ItemFilterInput) -> Self {
        ItemFilter {
            id: f.id.map(EqualFilter::from),
            name: f.name.map(SimpleStringFilter::from),
            code: f.code.map(SimpleStringFilter::from),
            is_visible: f.is_visible.and_then(|filter| filter.equal_to),
            r#type: None,
        }
    }
}

#[derive(Union)]
pub enum ItemsResponse {
    Response(ItemConnector),
}

pub fn items(
    ctx: &Context<'_>,
    page: Option<PaginationInput>,
    filter: Option<ItemFilterInput>,
    sort: Option<Vec<ItemSortInput>>,
) -> Result<ItemsResponse> {
    let connection_manager = ctx.get_connection_manager();
    let items = get_items(
        connection_manager,
        page.map(PaginationOption::from),
        filter.map(ItemFilter::from),
        // Currently only one sort option is supported, use the first from the list.
        sort.map(|mut sort_list| sort_list.pop())
            .flatten()
            .map(|sort| sort.to_domain()),
    )
    .map_err(StandardGraphqlError::from_list_error)?;

    Ok(ItemsResponse::Response(ItemConnector::from_domain(items)))
}

impl ItemSortInput {
    pub fn to_domain(self) -> ItemSort {
        use ItemSortField as to;
        use ItemSortFieldInput as from;
        let key = match self.key {
            from::Name => to::Name,
            from::Code => to::Code,
        };

        ItemSort {
            key,
            desc: self.desc,
        }
    }
}

#[cfg(test)]
mod test {
    use async_graphql::EmptyMutation;
    use graphql_core::{assert_graphql_query, test_helpers::setup_graphl_test};
    use repository::mock::MockDataInserts;
    use repository::mock::{mock_item_stats_item1, mock_item_stats_item2};
    use serde_json::json;

    use crate::GeneralQueries;

    #[actix_rt::test]
    async fn test_graphql_items_query() {
        let (_, _, _, settings) = setup_graphl_test(
            GeneralQueries,
            EmptyMutation,
            "test_items_query",
            MockDataInserts::all(),
        )
        .await;

        let query = r#"query items($itemFilter: ItemFilterInput!) {
            items(filter: $itemFilter) {
                ... on ItemConnector {
                  nodes {
                      id
                      name
                      code
                      isVisible
                      unitName
                      availableBatches(storeId: \"store_a\") {
                         ... on StockLineConnector {
                            nodes {
                                id
                            }
                          }
                      }
                  }
               }
            }
        }"#;

        let variables = json!({
            "itemFilter": {
                "name": {
                    "like": "item_query_test"
                }
            }
        });

        let expected = json!({
              "items": {
                  "nodes": [
                      {
                          "id": "item_query_test1",
                          "name": "name_item_query_test1",
                          "code": "code_item_query_test1",
                          "isVisible": true,
                          "unitName": null,
                          "availableBatches": {
                              "nodes": [ { "id": "item_query_test1" } ]
                          }
                      },
                      {
                          "id": "item_query_test2",
                          "name": "name_item_query_test2",
                          "code": "code_item_query_test2",
                          "isVisible": false,
                          "unitName": "name_item_query_test2",
                           "availableBatches": {
                              "nodes": []
                          }
                      }
                  ]
              }
          }
        );
        assert_graphql_query!(&settings, query, &Some(variables), &expected, None);
    }
    #[actix_rt::test]
    async fn test_graphql_item_stats_loader() {
        let (_, _, _, settings) = setup_graphl_test(
            GeneralQueries,
            EmptyMutation,
            "test_graphql_item_stats_loader",
            MockDataInserts::all(),
        )
        .await;

        let query = r#"
        query($filter: ItemFilterInput) {
          items(filter: $filter) {
            ... on ItemConnector {
              nodes {
                id
                stats(storeId: \"store_a\") {
                    averageMonthlyConsumption
                    availableMonthsOfStockOnHand
                    availableStockOnHand
                }
              }
            }
          }
       }
        "#;

        let variables = json!({
          "filter": {
            "id": {
                "equalAny": [&mock_item_stats_item1().id, &mock_item_stats_item2().id]
            },
          }
        }
        );

        // As per item stats repository test
        let expected = json!({
            "items": {
                "nodes": [{
                    "id": &mock_item_stats_item1().id,
                    "stats": {
                        "averageMonthlyConsumption":  15,
                        "availableStockOnHand": 210,
                        "availableMonthsOfStockOnHand": 210 as f64 / 15 as f64
                    }
                },
                {
                    "id": &mock_item_stats_item2().id,
                    "stats": {
                        "averageMonthlyConsumption": 5,
                        "availableStockOnHand": 22,
                        "availableMonthsOfStockOnHand": 22 as f64 / 5 as f64
                    },
                }]
            }
        }
        );

        assert_graphql_query!(&settings, query, &Some(variables.clone()), &expected, None);
    }
}
