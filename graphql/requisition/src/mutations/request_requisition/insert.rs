use async_graphql::*;
use graphql_core::{
    standard_graphql_error::validate_auth, standard_graphql_error::StandardGraphqlError, ContextExt,
};
use graphql_types::{
    generic_errors::OtherPartyNotASupplier,
    types::{NameNode, RequisitionNode},
};
use repository::Requisition;
use service::{
    permission_validation::{Resource, ResourceAccessRequest},
    requisition::request_requisition::{
        InsertRequestRequisition as ServiceInput, InsertRequestRequisitionError as ServiceError,
    },
};

#[derive(InputObject)]
#[graphql(name = "InsertRequestRequisitionInput")]
pub struct InsertInput {
    pub id: String,
    pub other_party_id: String,
    pub colour: Option<String>,
    pub their_reference: Option<String>,
    pub comment: Option<String>,
    pub max_months_of_stock: f64,
    pub min_months_of_stock: f64,
}

#[derive(Interface)]
#[graphql(name = "InsertRequestRequisitionErrorInterface")]
#[graphql(field(name = "description", type = "String"))]
pub enum InsertErrorInterface {
    OtherPartyNotASupplier(OtherPartyNotASupplier),
}

#[derive(SimpleObject)]
#[graphql(name = "InsertRequestRequisitionError")]
pub struct InsertError {
    pub error: InsertErrorInterface,
}

#[derive(Union)]
#[graphql(name = "InsertRequestRequisitionResponse")]
pub enum InsertResponse {
    Error(InsertError),
    Response(RequisitionNode),
}

pub fn insert(ctx: &Context<'_>, store_id: &str, input: InsertInput) -> Result<InsertResponse> {
    validate_auth(
        ctx,
        &ResourceAccessRequest {
            resource: Resource::EditRequisition,
            store_id: Some(store_id.to_string()),
        },
    )?;

    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;

    map_response(
        service_provider
            .requisition_service
            .insert_request_requisition(&service_context, store_id, input.to_domain()),
    )
}

pub fn map_response(from: Result<Requisition, ServiceError>) -> Result<InsertResponse> {
    let result = match from {
        Ok(requisition) => InsertResponse::Response(RequisitionNode::from_domain(requisition)),
        Err(error) => InsertResponse::Error(InsertError {
            error: map_error(error)?,
        }),
    };

    Ok(result)
}

impl InsertInput {
    pub fn to_domain(self) -> ServiceInput {
        let InsertInput {
            id,
            other_party_id,
            colour,
            their_reference,
            comment,
            max_months_of_stock,
            min_months_of_stock,
        } = self;

        ServiceInput {
            id,
            other_party_id,
            colour,
            their_reference,
            comment,
            max_months_of_stock,
            min_months_of_stock,
        }
    }
}

fn map_error(error: ServiceError) -> Result<InsertErrorInterface> {
    use StandardGraphqlError::*;
    let formatted_error = format!("{:#?}", error);

    let graphql_error = match error {
        // Structured Errors
        ServiceError::OtherPartyNotASupplier(name) => {
            return Ok(InsertErrorInterface::OtherPartyNotASupplier(
                OtherPartyNotASupplier(NameNode { name }),
            ))
        }
        // Standard Graphql Errors
        ServiceError::RequisitionAlreadyExists => BadUserInput(formatted_error),
        ServiceError::OtherPartyDoesNotExist => BadUserInput(formatted_error),
        ServiceError::OtherPartyIsThisStore => BadUserInput(formatted_error),
        ServiceError::OtherPartyIsNotAStore => BadUserInput(formatted_error),
        ServiceError::NewlyCreatedRequisitionDoesNotExist => InternalError(formatted_error),
        ServiceError::DatabaseError(_) => InternalError(formatted_error),
    };

    Err(graphql_error.extend())
}

#[cfg(test)]
mod test {
    use async_graphql::EmptyMutation;
    use graphql_core::{
        assert_graphql_query, assert_standard_graphql_error, test_helpers::setup_graphl_test,
    };
    use repository::Name;
    use repository::{
        mock::{mock_name_a, mock_request_draft_requisition, MockDataInserts},
        Requisition, StorageConnectionManager,
    };
    use serde_json::json;
    use service::{
        requisition::{
            request_requisition::{
                InsertRequestRequisition as ServiceInput,
                InsertRequestRequisitionError as ServiceError,
            },
            RequisitionServiceTrait,
        },
        service_provider::{ServiceContext, ServiceProvider},
    };

    use crate::RequisitionMutations;

    type InsertLineMethod =
        dyn Fn(&str, ServiceInput) -> Result<Requisition, ServiceError> + Sync + Send;

    pub struct TestService(pub Box<InsertLineMethod>);

    impl RequisitionServiceTrait for TestService {
        fn insert_request_requisition(
            &self,
            _: &ServiceContext,
            store_id: &str,
            input: ServiceInput,
        ) -> Result<Requisition, ServiceError> {
            self.0(store_id, input)
        }
    }

    fn service_provider(
        test_service: TestService,
        connection_manager: &StorageConnectionManager,
    ) -> ServiceProvider {
        let mut service_provider = ServiceProvider::new(connection_manager.clone());
        service_provider.requisition_service = Box::new(test_service);
        service_provider
    }

    fn empty_variables() -> serde_json::Value {
        json!({
          "input": {
            "id": "n/a",
            "otherPartyId": "n/a",
            "maxMonthsOfStock": 0,
            "minMonthsOfStock": 0
          },
          "storeId": "n/a"
        })
    }

    #[actix_rt::test]
    async fn test_graphql_insert_request_requisition_errors() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            RequisitionMutations,
            "test_graphql_insert_request_requisition_structured_errors",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: InsertRequestRequisitionInput!, $storeId: String) {
            insertRequestRequisition(storeId: $storeId, input: $input) {
              ... on InsertRequestRequisitionError {
                error {
                  __typename
                }
              }
            }
          }
        "#;

        // OtherPartyNotASupplier
        let test_service = TestService(Box::new(|_, _| {
            Err(ServiceError::OtherPartyNotASupplier(Name {
                name_row: mock_name_a(),
                name_store_join_row: None,
                store_row: None,
            }))
        }));

        let expected = json!({
            "insertRequestRequisition": {
              "error": {
                "__typename": "OtherPartyNotASupplier"
              }
            }
          }
        );

        assert_graphql_query!(
            &settings,
            mutation,
            &Some(empty_variables()),
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // RequisitionAlreadyExists
        let test_service =
            TestService(Box::new(|_, _| Err(ServiceError::RequisitionAlreadyExists)));
        let expected_message = "Bad user input";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        // OtherPartyDoesNotExist
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::OtherPartyDoesNotExist)));
        let expected_message = "Bad user input";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        // OtherPartyIsThisStore
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::OtherPartyIsThisStore)));
        let expected_message = "Bad user input";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        // OtherPartyIsNotAStore
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::OtherPartyIsNotAStore)));
        let expected_message = "Bad user input";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        // OtherPartyIsNotAStore
        let test_service = TestService(Box::new(|_, _| {
            Err(ServiceError::NewlyCreatedRequisitionDoesNotExist)
        }));
        let expected_message = "Internal error";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );
    }

    #[actix_rt::test]
    async fn test_graphql_insert_request_requisition_success() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            RequisitionMutations,
            "test_graphql_insert_request_requisition_success",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($storeId: String, $input: InsertRequestRequisitionInput!) {
            insertRequestRequisition(storeId: $storeId, input: $input) {
                ... on RequisitionNode {
                    id
                }
            }
          }
        "#;

        // Success
        let test_service = TestService(Box::new(|store_id, input| {
            assert_eq!(store_id, "store_a");
            assert_eq!(
                input,
                ServiceInput {
                    id: "id input".to_string(),
                    other_party_id: "other party input".to_string(),
                    colour: Some("colour input".to_string()),
                    their_reference: Some("reference input".to_string()),
                    comment: Some("comment input".to_string()),
                    max_months_of_stock: 1.0,
                    min_months_of_stock: 2.0
                }
            );
            Ok(Requisition {
                requisition_row: mock_request_draft_requisition(),
                name_row: mock_name_a(),
            })
        }));

        let variables = json!({
          "input": {
            "id": "id input",
            "otherPartyId": "other party input",
            "maxMonthsOfStock": 1,
            "minMonthsOfStock": 2,
            "colour": "colour input",
            "theirReference": "reference input",
            "comment": "comment input",
          },
          "storeId": "store_a"
        });

        let expected = json!({
            "insertRequestRequisition": {
                "id": mock_request_draft_requisition().id
            }
          }
        );

        assert_graphql_query!(
            &settings,
            mutation,
            &Some(variables),
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );
    }
}
