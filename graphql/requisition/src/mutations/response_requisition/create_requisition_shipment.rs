use async_graphql::*;
use graphql_core::{
    simple_generic_errors::{CannotEditRequisition, RecordDoesNotExist},
    standard_graphql_error::validate_auth,
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};
use graphql_types::types::InvoiceNode;
use service::{
    permission_validation::{Resource, ResourceAccessRequest},
    requisition::response_requisition::{
        CreateRequisitionShipment as ServiceInput, CreateRequisitionShipmentError as ServiceError,
    },
};

#[derive(InputObject)]
pub struct CreateRequisitionShipmentInput {
    pub response_requisition_id: String,
}

#[derive(Interface)]
#[graphql(name = "CreateRequisitionShipmentErrorInterface")]
#[graphql(field(name = "description", type = "String"))]
pub enum DeleteErrorInterface {
    RecordDoesNotExist(RecordDoesNotExist),
    NothingRemainingToSupply(NothingRemainingToSupply),
    CannotEditRequisition(CannotEditRequisition),
}

#[derive(SimpleObject)]
#[graphql(name = "CreateRequisitionShipmentError")]
pub struct DeleteError {
    pub error: DeleteErrorInterface,
}

#[derive(Union)]
#[graphql(name = "CreateRequisitionShipmentResponse")]
pub enum CreateRequisitionShipmentResponse {
    Error(DeleteError),
    Response(InvoiceNode),
}

pub fn create_requisition_shipment(
    ctx: &Context<'_>,
    store_id: &str,
    input: CreateRequisitionShipmentInput,
) -> Result<CreateRequisitionShipmentResponse> {
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
        .create_requisition_shipment(&service_context, store_id, input.to_domain())
    {
        Ok(invoice) => {
            CreateRequisitionShipmentResponse::Response(InvoiceNode::from_domain(invoice))
        }
        Err(error) => CreateRequisitionShipmentResponse::Error(DeleteError {
            error: map_error(error)?,
        }),
    };

    Ok(response)
}

impl CreateRequisitionShipmentInput {
    pub fn to_domain(self) -> ServiceInput {
        let CreateRequisitionShipmentInput {
            response_requisition_id,
        } = self;
        ServiceInput {
            response_requisition_id,
        }
    }
}

fn map_error(error: ServiceError) -> Result<DeleteErrorInterface> {
    use StandardGraphqlError::*;
    let formatted_error = format!("{:#?}", error);

    let graphql_error = match error {
        // Structured Errors
        ServiceError::RequisitionDoesNotExist => {
            return Ok(DeleteErrorInterface::RecordDoesNotExist(
                RecordDoesNotExist {},
            ))
        }
        ServiceError::CannotEditRequisition => {
            return Ok(DeleteErrorInterface::CannotEditRequisition(
                CannotEditRequisition {},
            ))
        }
        ServiceError::NothingRemainingToSupply => {
            return Ok(DeleteErrorInterface::NothingRemainingToSupply(
                NothingRemainingToSupply {},
            ))
        }
        // Standard Graphql Errors
        ServiceError::NotThisStoreRequisition => BadUserInput(formatted_error),
        ServiceError::NotAResponseRequisition => BadUserInput(formatted_error),
        ServiceError::CreatedInvoiceDoesNotExist => InternalError(formatted_error),
        ServiceError::ProblemGettingOtherParty => InternalError(formatted_error),
        ServiceError::ProblemFindingItem => InternalError(formatted_error),
        ServiceError::DatabaseError(_) => InternalError(formatted_error),
    };

    Err(graphql_error.extend())
}

pub struct NothingRemainingToSupply;
#[Object]
impl NothingRemainingToSupply {
    pub async fn description(&self) -> &'static str {
        "Requisition is fulfilled, check associated invoices and supply quantity"
    }
}

#[cfg(test)]
mod test {
    use async_graphql::EmptyMutation;
    use graphql_core::{
        assert_graphql_query, assert_standard_graphql_error, test_helpers::setup_graphl_test,
    };
    use repository::{
        mock::{mock_name_store_a, mock_outbound_shipment_a, mock_store_a, MockDataInserts},
        Invoice, StorageConnectionManager,
    };
    use serde_json::json;
    use service::{
        requisition::{
            response_requisition::{
                CreateRequisitionShipment as ServiceInput,
                CreateRequisitionShipmentError as ServiceError,
            },
            RequisitionServiceTrait,
        },
        service_provider::{ServiceContext, ServiceProvider},
    };

    use crate::RequisitionMutations;

    type DeleteLineMethod =
        dyn Fn(&str, ServiceInput) -> Result<Invoice, ServiceError> + Sync + Send;

    pub struct TestService(pub Box<DeleteLineMethod>);

    impl RequisitionServiceTrait for TestService {
        fn create_requisition_shipment(
            &self,
            _: &ServiceContext,
            store_id: &str,
            input: ServiceInput,
        ) -> Result<Invoice, ServiceError> {
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
            "responseRequisitionId": "n/a"
          },
          "storeId": "n/a"
        })
    }

    #[actix_rt::test]
    async fn test_graphql_create_requisition_shipment_errors() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            RequisitionMutations,
            "test_graphql_create_requisition_shipment_structured_errors",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: CreateRequisitionShipmentInput!, $storeId: String) {
            createRequisitionShipment(storeId: $storeId, input: $input) {
              ... on CreateRequisitionShipmentError {
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
            "createRequisitionShipment": {
              "error": {
                "__typename": "RecordDoesNotExist"
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
            "createRequisitionShipment": {
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

        // NothingRemainingToSupply
        let test_service =
            TestService(Box::new(|_, _| Err(ServiceError::NothingRemainingToSupply)));

        let expected = json!({
            "createRequisitionShipment": {
              "error": {
                "__typename": "NothingRemainingToSupply"
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

        // ProblemGettingOtherParty
        let test_service =
            TestService(Box::new(|_, _| Err(ServiceError::ProblemGettingOtherParty)));
        let expected_message = "Internal error";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        // ProblemFindingItem
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::ProblemFindingItem)));
        let expected_message = "Internal error";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        // CreatedInvoiceDoesNotExist
        let test_service = TestService(Box::new(|_, _| {
            Err(ServiceError::CreatedInvoiceDoesNotExist)
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
    async fn test_graphql_create_requisition_shipment_success() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            RequisitionMutations,
            "test_graphql_create_requisition_shipment_success",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($storeId: String, $input: CreateRequisitionShipmentInput!) {
            createRequisitionShipment(storeId: $storeId, input: $input) {
                ... on InvoiceNode{
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
                    response_requisition_id: "id input".to_string(),
                }
            );
            Ok(Invoice {
                invoice_row: mock_outbound_shipment_a(),
                name_row: mock_name_store_a(),
                store_row: mock_store_a(),
            })
        }));

        let variables = json!({
          "input": {
            "responseRequisitionId": "id input"
          },
          "storeId": "store_a"
        });

        let expected = json!({
            "createRequisitionShipment": {
              "id": mock_outbound_shipment_a().id
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
