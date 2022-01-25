mod graphql {
    use crate::graphql::assert_graphql_query;
    use chrono::NaiveDate;
    use repository::{
        mock::MockDataInserts,
        schema::{StockTakeRow, StockTakeStatus},
        StorageConnectionManager,
    };
    use serde_json::json;
    use server::test_utils::setup_all;
    use service::{
        service_provider::{ServiceContext, ServiceProvider},
        stock_take::{
            update::{UpdateStockTakeError, UpdateStockTakeInput},
            StockTakeServiceTrait,
        },
    };

    type UpdateMethod = dyn Fn(
            &ServiceContext,
            &str,
            UpdateStockTakeInput,
        ) -> Result<StockTakeRow, UpdateStockTakeError>
        + Sync
        + Send;

    pub struct TestService(pub Box<UpdateMethod>);

    impl StockTakeServiceTrait for TestService {
        fn update_stock_take(
            &self,
            ctx: &ServiceContext,
            store_id: &str,
            input: UpdateStockTakeInput,
        ) -> Result<StockTakeRow, UpdateStockTakeError> {
            (self.0)(ctx, store_id, input)
        }
    }

    pub fn service_provider(
        test_service: TestService,
        connection_manager: &StorageConnectionManager,
    ) -> ServiceProvider {
        let mut service_provider = ServiceProvider::new(connection_manager.clone());
        service_provider.stock_take_service = Box::new(test_service);
        service_provider
    }

    #[actix_rt::test]
    async fn test_graphql_stock_take_update() {
        let (_, _, connection_manager, settings) = setup_all(
            "omsupply-database-gql-stock_take_update",
            MockDataInserts::all(),
        )
        .await;

        let query = r#"mutation UpdateStockTake($storeId: String, $input: UpdateStockTakeInput!) {
            updateStockTake(storeId: $storeId, input: $input) {
                ... on UpdateStockTakeError {
                  error {
                    __typename
                  }
                }
                ... on StockTakeNode {                    
                        id
                        storeId
                        comment
                        description
                        status
                        createdDatetime
                        finalisedDatetime
                        inventoryAdjustmentId
                }
            }
        }"#;

        // SnapshotCountCurrentCountMismatch
        let test_service = TestService(Box::new(|_, _, _| {
            Err(UpdateStockTakeError::SnapshotCountCurrentCountMismatch(
                vec![],
            ))
        }));
        let variables = Some(json!({
            "storeId": "store id",
            "input": {
                "id": "stock take id"
            }
        }));
        let expected = json!({
            "updateStockTake": {
              "error": {
                "__typename": "SnapshotCountCurrentCountMismatch"
              }
            }
          }
        );
        assert_graphql_query!(
            &settings,
            query,
            &variables,
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // success
        let test_service = TestService(Box::new(|_, _, _| {
            Ok(StockTakeRow {
                id: "id1".to_string(),
                store_id: "store id".to_string(),
                comment: Some("comment".to_string()),
                description: Some("description".to_string()),
                status: StockTakeStatus::Finalised,
                created_datetime: NaiveDate::from_ymd(2022, 1, 22).and_hms(15, 16, 0),
                finalised_datetime: Some(NaiveDate::from_ymd(2022, 1, 23).and_hms(15, 16, 0)),
                inventory_adjustment_id: Some("inv id".to_string()),
            })
        }));
        let variables = Some(json!({
            "storeId": "store id",
            "input": {
                "id": "id1"
            }
        }));
        let expected = json!({
            "updateStockTake": {
              "id": "id1",
              "storeId": "store id",
              "comment": "comment",
              "description": "description",
              "status": "FINALISED",
              "createdDatetime": "2022-01-22T15:16:00",
              "finalisedDatetime": "2022-01-23T15:16:00",
              "inventoryAdjustmentId": "inv id",
            }
          }
        );
        assert_graphql_query!(
            &settings,
            query,
            &variables,
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );
    }
}