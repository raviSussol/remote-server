use async_graphql::*;
use chrono::NaiveDate;

use graphql_core::simple_generic_errors::CannotEditStocktake;
use graphql_core::standard_graphql_error::{validate_auth, StandardGraphqlError};
use graphql_core::ContextExt;
use graphql_types::types::StocktakeNode;
use repository::Stocktake;
use service::{
    auth::{Resource, ResourceAccessRequest},
    stocktake::{InsertStocktake as ServiceInput, InsertStocktakeError as ServiceError},
};

#[derive(InputObject)]
#[graphql(name = "InsertStocktakeInput")]
pub struct InsertInput {
    pub id: String,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub is_locked: Option<bool>,
    pub stocktake_date: Option<NaiveDate>,
}

#[derive(Union)]
#[graphql(name = "InsertStocktakeResponse")]
pub enum InsertResponse {
    Response(StocktakeNode),
}

#[derive(Interface)]
#[graphql(name = "InsertStocktakeErrorInterface")]
#[graphql(field(name = "description", type = "String"))]
pub enum InsertErrorInterface {
    CannotEditStocktake(CannotEditStocktake),
}

#[derive(SimpleObject)]
#[graphql(name = "InsertStocktakeError")]
pub struct InsertError {
    pub error: InsertErrorInterface,
}

pub fn insert(ctx: &Context<'_>, store_id: &str, input: InsertInput) -> Result<InsertResponse> {
    let user = validate_auth(
        ctx,
        &ResourceAccessRequest {
            resource: Resource::MutateStocktake,
            store_id: Some(store_id.to_string()),
        },
    )?;

    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;
    map_response(service_provider.stocktake_service.insert_stocktake(
        &service_context,
        store_id,
        &user.user_id,
        input.to_domain(),
    ))
}

pub fn map_response(from: Result<Stocktake, ServiceError>) -> Result<InsertResponse> {
    match from {
        Ok(stocktake) => Ok(InsertResponse::Response(StocktakeNode::from_domain(
            stocktake,
        ))),
        Err(error) => {
            use StandardGraphqlError::*;
            let formatted_error = format!("{:#?}", error);

            let graphql_error = match error {
                ServiceError::InvalidStore => BadUserInput(formatted_error),
                ServiceError::StocktakeAlreadyExists => BadUserInput(formatted_error),
                ServiceError::InternalError(err) => InternalError(err),
                ServiceError::DatabaseError(_) => InternalError(formatted_error),
            };

            Err(graphql_error.extend())
        }
    }
}

impl InsertInput {
    pub fn to_domain(self) -> ServiceInput {
        let InsertInput {
            id,
            comment,
            description,
            stocktake_date,
            is_locked,
        } = self;

        ServiceInput {
            id,
            comment,
            description,
            stocktake_date,
            is_locked,
        }
    }
}

#[cfg(test)]
mod test {
    use async_graphql::EmptyMutation;
    use chrono::NaiveDate;
    use graphql_core::{assert_graphql_query, test_helpers::setup_graphl_test};
    use repository::{mock::MockDataInserts, Stocktake, StocktakeRow, StorageConnectionManager};
    use serde_json::json;
    use service::{
        service_provider::{ServiceContext, ServiceProvider},
        stocktake::{
            StocktakeServiceTrait, {InsertStocktake, InsertStocktakeError},
        },
    };
    use util::inline_init;

    use crate::StocktakeMutations;

    type ServiceMethod = dyn Fn(&ServiceContext, &str, InsertStocktake) -> Result<Stocktake, InsertStocktakeError>
        + Sync
        + Send;

    pub struct TestService(pub Box<ServiceMethod>);

    impl StocktakeServiceTrait for TestService {
        fn insert_stocktake(
            &self,
            ctx: &ServiceContext,
            store_id: &str,
            _: &str,
            input: InsertStocktake,
        ) -> Result<Stocktake, InsertStocktakeError> {
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
    async fn test_graphql_stocktake_insert() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            StocktakeMutations,
            "omsupply-database-gql-stocktake_insert",
            MockDataInserts::all(),
        )
        .await;

        let query = r#"mutation InsertStocktake($storeId: String, $input: InsertStocktakeInput!) {
            insertStocktake(storeId: $storeId, input: $input) {
                ... on StocktakeNode {                    
                        id
                }
            }
        }"#;

        // success
        let test_service = TestService(Box::new(|_, _, input| {
            assert_eq!(
                input,
                InsertStocktake {
                    id: "id1".to_string(),
                    comment: Some("comment".to_string()),
                    description: Some("description".to_string()),
                    stocktake_date: Some(NaiveDate::from_ymd(2022, 01, 03)),
                    is_locked: Some(true)
                }
            );
            // StocktakeNode result is checked in queries
            Ok(inline_init(|r: &mut StocktakeRow| r.id = "id1".to_string()))
        }));
        let variables = Some(json!({
            "storeId": "store id",
            "input": {
              "id": "id1",
              "comment": "comment",
              "description": "description",
              "stocktakeDate": "2022-01-03",
              "isLocked": true
            }
        }));
        let expected = json!({
            "insertStocktake": {
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
