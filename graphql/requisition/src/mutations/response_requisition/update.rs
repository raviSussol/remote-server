use async_graphql::*;

use graphql_core::{
    simple_generic_errors::{CannotEditRequisition, RecordNotFound},
    standard_graphql_error::validate_auth,
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};
use graphql_types::types::RequisitionNode;
use service::{
    permission_validation::{Resource, ResourceAccessRequest},
    requisition::response_requisition::{
        UpdateResponseRequisition as ServiceInput, UpdateResponseRequisitionError as ServiceError,
        UpdateResponseRequstionStatus,
    },
};

#[derive(InputObject)]
#[graphql(name = "UpdateResponseRequisitionInput")]
pub struct UpdateInput {
    pub id: String,
    pub colour: Option<String>,
    pub their_reference: Option<String>,
    pub comment: Option<String>,
    pub status: Option<UpdateResponseRequisitionStatusInput>,
}

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug)]
pub enum UpdateResponseRequisitionStatusInput {
    Finalised,
}

#[derive(Interface)]
#[graphql(name = "UpdateResponseRequisitionErrorInterface")]
#[graphql(field(name = "description", type = "String"))]
pub enum UpdateErrorInterface {
    RecordNotFound(RecordNotFound),
    CannotEditRequisition(CannotEditRequisition),
}

#[derive(SimpleObject)]
#[graphql(name = "UpdateResponseRequisitionError")]
pub struct UpdateError {
    pub error: UpdateErrorInterface,
}

#[derive(Union)]
#[graphql(name = "UpdateResponseRequisitionResponse")]
pub enum UpdateResponse {
    Error(UpdateError),
    Response(RequisitionNode),
}

pub fn update(ctx: &Context<'_>, store_id: &str, input: UpdateInput) -> Result<UpdateResponse> {
    validate_auth(
        ctx,
        &ResourceAccessRequest {
            resource: Resource::EditRequisition,
            store_id: Some(store_id.to_string()),
        },
    )?;

    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;

    let response = match service_provider
        .requisition_service
        .update_response_requisition(&service_context, store_id, input.to_domain())
    {
        Ok(requisition) => UpdateResponse::Response(RequisitionNode::from_domain(requisition)),
        Err(error) => UpdateResponse::Error(UpdateError {
            error: map_error(error)?,
        }),
    };

    Ok(response)
}

impl UpdateInput {
    pub fn to_domain(self) -> ServiceInput {
        let UpdateInput {
            id,
            colour,
            their_reference,
            comment,
            status,
        } = self;

        ServiceInput {
            id,
            colour,
            their_reference,
            comment,
            status: status.map(|status| status.to_domain()),
        }
    }
}

fn map_error(error: ServiceError) -> Result<UpdateErrorInterface> {
    use StandardGraphqlError::*;
    let formatted_error = format!("{:#?}", error);

    let graphql_error = match error {
        // Structured Errors
        ServiceError::RequisitionDoesNotExist => {
            return Ok(UpdateErrorInterface::RecordNotFound(
                RecordNotFound {},
            ))
        }
        ServiceError::CannotEditRequisition => {
            return Ok(UpdateErrorInterface::CannotEditRequisition(
                CannotEditRequisition {},
            ))
        }
        // Standard Graphql Errors
        ServiceError::NotThisStoreRequisition => BadUserInput(formatted_error),
        ServiceError::NotAResponseRequisition => BadUserInput(formatted_error),
        ServiceError::UpdatedRequisitionDoesNotExist => InternalError(formatted_error),
        ServiceError::DatabaseError(_) => InternalError(formatted_error),
    };

    Err(graphql_error.extend())
}

impl UpdateResponseRequisitionStatusInput {
    pub fn to_domain(self) -> UpdateResponseRequstionStatus {
        use UpdateResponseRequisitionStatusInput::*;
        match self {
            Finalised => UpdateResponseRequstionStatus::Finalised,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::RequisitionMutations;
    use async_graphql::EmptyMutation;
    use graphql_core::{
        assert_graphql_query, assert_standard_graphql_error, test_helpers::setup_graphl_test,
    };
    use repository::{
        mock::{mock_name_a, mock_request_draft_requisition, MockDataInserts},
        Requisition, StorageConnectionManager,
    };
    use serde_json::json;

    use service::{
        requisition::{
            response_requisition::{
                UpdateResponseRequisition as ServiceInput,
                UpdateResponseRequisitionError as ServiceError, UpdateResponseRequstionStatus,
            },
            RequisitionServiceTrait,
        },
        service_provider::{ServiceContext, ServiceProvider},
    };

    type UpdateLineMethod =
        dyn Fn(&str, ServiceInput) -> Result<Requisition, ServiceError> + Sync + Send;

    pub struct TestService(pub Box<UpdateLineMethod>);

    impl RequisitionServiceTrait for TestService {
        fn update_response_requisition(
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
          },
          "storeId": "n/a"
        })
    }

    #[actix_rt::test]
    async fn test_graphql_update_response_requisition_errors() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            RequisitionMutations,
            "test_graphql_update_response_requisition_structured_errors",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: UpdateResponseRequisitionInput!, $storeId: String) {
            updateResponseRequisition(storeId: $storeId, input: $input) {
              ... on UpdateResponseRequisitionError {
                error {
                  __typename
                }
              }
            }
          }
        "#;

        // RequisitionDoesNotExist
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::RequisitionDoesNotExist)));

        let expected = json!({
            "updateResponseRequisition": {
              "error": {
                "__typename": "RecordNotFound"
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

        // CannotEditRequisition
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::CannotEditRequisition)));

        let expected = json!({
            "updateResponseRequisition": {
              "error": {
                "__typename": "CannotEditRequisition"
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

        // NotThisStoreRequisition
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::NotThisStoreRequisition)));
        let expected_message = "Bad user input";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        // NotAResponseRequisition
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::NotAResponseRequisition)));
        let expected_message = "Bad user input";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        // UpdatedRequisitionDoesNotExist
        let test_service = TestService(Box::new(|_, _| {
            Err(ServiceError::UpdatedRequisitionDoesNotExist)
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
    async fn test_graphql_update_response_requisition_success() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            RequisitionMutations,
            "test_graphql_update_response_requisition_success",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($storeId: String, $input: UpdateResponseRequisitionInput!) {
            updateResponseRequisition(storeId: $storeId, input: $input) {
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
                    colour: Some("colour input".to_string()),
                    their_reference: Some("reference input".to_string()),
                    comment: Some("comment input".to_string()),
                    status: Some(UpdateResponseRequstionStatus::Finalised)
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

            "colour": "colour input",
            "theirReference": "reference input",
            "comment": "comment input",
            "status": "FINALISED"
          },
          "storeId": "store_a"
        });

        let expected = json!({
            "updateResponseRequisition": {
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
