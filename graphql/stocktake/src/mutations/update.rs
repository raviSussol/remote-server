use async_graphql::*;

use chrono::NaiveDate;
use graphql_core::simple_generic_errors::{CannotEditStocktake, StocktakeIsLocked};
use graphql_core::standard_graphql_error::{validate_auth, StandardGraphqlError};
use graphql_core::ContextExt;
use graphql_types::types::{StocktakeLineConnector, StocktakeNode, StocktakeNodeStatus};
use repository::{Stocktake, StocktakeLine};
use service::{
    permission_validation::{Resource, ResourceAccessRequest},
    stocktake::{UpdateStocktake as ServiceInput, UpdateStocktakeError as ServiceError},
};

#[derive(InputObject)]
#[graphql(name = "UpdateStocktakeInput")]
pub struct UpdateInput {
    pub id: String,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub status: Option<StocktakeNodeStatus>,
    pub stocktake_date: Option<NaiveDate>,
    pub is_locked: Option<bool>,
}

pub struct SnapshotCountCurrentCountMismatch(Vec<StocktakeLine>);
#[Object]
impl SnapshotCountCurrentCountMismatch {
    pub async fn description(&self) -> &'static str {
        "Snapshot count doesn't match the current stock count"
    }

    pub async fn lines(&self) -> StocktakeLineConnector {
        StocktakeLineConnector::from_domain_vec(self.0.clone())
    }
}

#[derive(Interface)]
#[graphql(name = "UpdateStocktakeErrorInterface")]
#[graphql(field(name = "description", type = "String"))]
pub enum UpdateErrorInterface {
    SnapshotCountCurrentCountMismatch(SnapshotCountCurrentCountMismatch),
    StocktakeIsLocked(StocktakeIsLocked),
    CannotEditStocktake(CannotEditStocktake),
}

#[derive(SimpleObject)]
#[graphql(name = "UpdateStocktakeError")]
pub struct UpdateError {
    pub error: UpdateErrorInterface,
}

#[derive(Union)]
#[graphql(name = "UpdateStocktakeResponse")]
pub enum UpdateResponse {
    Error(UpdateError),
    Response(StocktakeNode),
}

pub fn update(ctx: &Context<'_>, store_id: &str, input: UpdateInput) -> Result<UpdateResponse> {
    let user = validate_auth(
        ctx,
        &ResourceAccessRequest {
            resource: Resource::MutateStocktake,
            store_id: Some(store_id.to_string()),
        },
    )?;

    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;
    map_response(service_provider.stocktake_service.update_stocktake(
        &service_context,
        store_id,
        &user.user_id,
        input.to_domain(),
    ))
}

pub fn map_response(from: Result<Stocktake, ServiceError>) -> Result<UpdateResponse> {
    let result = match from {
        Ok(stocktake) => UpdateResponse::Response(StocktakeNode::from_domain(stocktake)),
        Err(error) => UpdateResponse::Error(UpdateError {
            error: map_error(error)?,
        }),
    };

    Ok(result)
}

fn map_error(err: ServiceError) -> Result<UpdateErrorInterface> {
    use StandardGraphqlError::*;
    let formatted_error = format!("{:#?}", err);
    let graphql_error = match err {
        // Structured Errors
        ServiceError::SnapshotCountCurrentCountMismatch(lines) => {
            return Ok(UpdateErrorInterface::SnapshotCountCurrentCountMismatch(
                SnapshotCountCurrentCountMismatch(lines),
            ))
        }
        ServiceError::StocktakeIsLocked => {
            return Ok(UpdateErrorInterface::StocktakeIsLocked(
                StocktakeIsLocked {},
            ))
        }
        ServiceError::CannotEditFinalised => {
            return Ok(UpdateErrorInterface::CannotEditStocktake(
                CannotEditStocktake {},
            ))
        }
        // Standard Graphql Errors
        // TODO some are structured errors (where can be changed concurrently)
        ServiceError::InvalidStore => BadUserInput(formatted_error),
        ServiceError::StocktakeDoesNotExist => BadUserInput(formatted_error),
        ServiceError::NoLines => BadUserInput(formatted_error),
        ServiceError::InternalError(err) => InternalError(err),
        ServiceError::DatabaseError(_) => InternalError(formatted_error),
    };

    Err(graphql_error.extend())
}

impl UpdateInput {
    pub fn to_domain(self) -> ServiceInput {
        let UpdateInput {
            id,
            comment,
            description,
            status,
            is_locked,
            stocktake_date,
        } = self;

        ServiceInput {
            id,
            comment,
            description,
            status: status.map(|status| status.to_domain()),
            is_locked,
            stocktake_date,
        }
    }
}

#[cfg(test)]
mod graphql {
    use async_graphql::EmptyMutation;
    use chrono::NaiveDate;
    use graphql_core::{assert_graphql_query, test_helpers::setup_graphl_test};
    use repository::{
        mock::MockDataInserts, StocktakeRow, StocktakeStatus, StorageConnectionManager,
    };
    use serde_json::json;
    use service::{
        service_provider::{ServiceContext, ServiceProvider},
        stocktake::*,
    };

    use crate::StocktakeMutations;

    type UpdateMethod = dyn Fn(&ServiceContext, &str, UpdateStocktake) -> Result<StocktakeRow, UpdateStocktakeError>
        + Sync
        + Send;

    pub struct TestService(pub Box<UpdateMethod>);

    impl StocktakeServiceTrait for TestService {
        fn update_stocktake(
            &self,
            ctx: &ServiceContext,
            store_id: &str,
            _: &str,
            input: UpdateStocktake,
        ) -> Result<StocktakeRow, UpdateStocktakeError> {
            (self.0)(ctx, store_id, input)
        }
    }

    pub fn service_provider(
        test_service: TestService,
        connection_manager: &StorageConnectionManager,
    ) -> ServiceProvider {
        let mut service_provider = ServiceProvider::new(connection_manager.clone());
        service_provider.stocktake_service = Box::new(test_service);
        service_provider
    }

    #[actix_rt::test]
    async fn test_graphql_stocktake_update() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            StocktakeMutations,
            "omsupply-database-gql-stocktake_update",
            MockDataInserts::all(),
        )
        .await;

        let query = r#"mutation UpdateStocktake($storeId: String, $input: UpdateStocktakeInput!) {
            updateStocktake(storeId: $storeId, input: $input) {
                ... on UpdateStocktakeError {
                  error {
                    __typename
                  }
                }
                ... on StocktakeNode {                    
                        id
                }
            }
        }"#;

        let variables = Some(json!({
            "storeId": "store id",
            "input": {
                "id": "stocktake id"
            }
        }));

        // Stocktake is locked mapping
        let test_service = TestService(Box::new(|_, _, _| {
            Err(UpdateStocktakeError::StocktakeIsLocked)
        }));

        let expected = json!({
            "updateStocktake": {
              "error": {
                "__typename": "StocktakeIsLocked"
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

        // SnapshotCountCurrentCountMismatch
        let test_service = TestService(Box::new(|_, _, _| {
            Err(UpdateStocktakeError::SnapshotCountCurrentCountMismatch(
                vec![],
            ))
        }));

        let expected = json!({
            "updateStocktake": {
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
            Ok(StocktakeRow {
                id: "id1".to_string(),
                user_id: "".to_string(),
                stocktake_number: 123,
                store_id: "store id".to_string(),
                comment: Some("comment".to_string()),
                description: Some("description".to_string()),
                status: StocktakeStatus::Finalised,
                created_datetime: NaiveDate::from_ymd(2022, 1, 22).and_hms(15, 16, 0),
                stocktake_date: Some(NaiveDate::from_ymd(2022, 01, 24)),
                finalised_datetime: Some(NaiveDate::from_ymd(2022, 1, 23).and_hms(15, 16, 0)),
                inventory_adjustment_id: Some("inv id".to_string()),
                is_locked: false,
            })
        }));

        let expected = json!({
            "updateStocktake": {
              "id": "id1",
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
