use async_graphql::*;
use graphql_core::{
    simple_generic_errors::RecordNotFound,
    standard_graphql_error::{validate_auth, StandardGraphqlError},
    ContextExt,
};
use graphql_types::types::InvoiceLineNode;
use repository::InvoiceLine;
use service::{
    auth::{Resource, ResourceAccessRequest},
    invoice_line::outbound_shipment_unallocated_line::{
        UpdateOutboundShipmentUnallocatedLine as ServiceInput,
        UpdateOutboundShipmentUnallocatedLineError as ServiceError,
    },
};

#[derive(InputObject)]
#[graphql(name = "UpdateOutboundShipmentUnallocatedLineInput")]
pub struct UpdateInput {
    pub id: String,
    pub quantity: u32,
}

#[derive(Interface)]
#[graphql(name = "UpdateOutboundShipmentUnallocatedLineErrorInterface")]
#[graphql(field(name = "description", type = "String"))]
pub enum UpdateErrorInterface {
    RecordNotFound(RecordNotFound),
}

#[derive(SimpleObject)]
#[graphql(name = "UpdateOutboundShipmentUnallocatedLineError")]
pub struct UpdateError {
    pub error: UpdateErrorInterface,
}

#[derive(Union)]
#[graphql(name = "UpdateOutboundShipmentUnallocatedLineResponse")]
pub enum UpdateResponse {
    Error(UpdateError),
    Response(InvoiceLineNode),
}

impl UpdateInput {
    pub fn to_domain(self) -> ServiceInput {
        let UpdateInput { id, quantity } = self;

        ServiceInput { id, quantity }
    }
}

pub fn update(ctx: &Context<'_>, store_id: &str, input: UpdateInput) -> Result<UpdateResponse> {
    validate_auth(
        ctx,
        &ResourceAccessRequest {
            resource: Resource::MutateOutboundShipment,
            store_id: Some(store_id.to_string()),
        },
    )?;

    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;

    map_response(
        service_provider
            .invoice_line_service
            .update_outbound_shipment_unallocated_line(
                &service_context,
                store_id,
                input.to_domain(),
            ),
    )
}

pub fn map_response(from: Result<InvoiceLine, ServiceError>) -> Result<UpdateResponse> {
    let result = match from {
        Ok(invoice_line) => UpdateResponse::Response(InvoiceLineNode::from_domain(invoice_line)),
        Err(error) => UpdateResponse::Error(UpdateError {
            error: map_error(error)?,
        }),
    };

    Ok(result)
}

fn map_error(error: ServiceError) -> Result<UpdateErrorInterface> {
    use StandardGraphqlError::*;
    let formatted_error = format!("{:#?}", error);

    let graphql_error = match error {
        // Structured Errors
        ServiceError::LineDoesNotExist => {
            return Ok(UpdateErrorInterface::RecordNotFound(RecordNotFound {}))
        }
        // Standard Graphql Errors
        ServiceError::LineIsNotUnallocatedLine => BadUserInput(formatted_error),
        ServiceError::UpdatedLineDoesNotExist => InternalError(formatted_error),
        ServiceError::DatabaseError(_) => InternalError(formatted_error),
    };

    Err(graphql_error.extend())
}

#[cfg(test)]
mod graphql {
    use async_graphql::EmptyMutation;
    use graphql_core::{
        assert_graphql_query, assert_standard_graphql_error, test_helpers::setup_graphl_test,
    };
    use repository::{
        mock::{mock_outbound_shipment_a, mock_outbound_shipment_a_invoice_lines, MockDataInserts},
        InvoiceLine, StorageConnectionManager,
    };
    use serde_json::json;

    use service::{
        invoice_line::{
            outbound_shipment_unallocated_line::{
                UpdateOutboundShipmentUnallocatedLine as ServiceInput,
                UpdateOutboundShipmentUnallocatedLineError as ServiceError,
            },
            InvoiceLineServiceTrait,
        },
        service_provider::{ServiceContext, ServiceProvider},
    };

    use crate::InvoiceLineMutations;

    type UpdateLineMethod = dyn Fn(ServiceInput) -> Result<InvoiceLine, ServiceError> + Sync + Send;

    pub struct TestService(pub Box<UpdateLineMethod>);

    impl InvoiceLineServiceTrait for TestService {
        fn update_outbound_shipment_unallocated_line(
            &self,
            _: &ServiceContext,
            _: &str,
            input: ServiceInput,
        ) -> Result<InvoiceLine, ServiceError> {
            self.0(input)
        }
    }

    fn service_provider(
        test_service: TestService,
        connection_manager: &StorageConnectionManager,
    ) -> ServiceProvider {
        let mut service_provider = ServiceProvider::new(connection_manager.clone());
        service_provider.invoice_line_service = Box::new(test_service);
        service_provider
    }

    fn empty_variables() -> serde_json::Value {
        json!({
          "input": {
            "id": "n/a",
            "quantity": 0,
          }
        })
    }

    #[actix_rt::test]
    async fn test_graphql_update_unallocated_structured_errors() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            InvoiceLineMutations,
            "test_graphql_update_unallocated_line_structured_errors",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: UpdateOutboundShipmentUnallocatedLineInput!) {
            updateOutboundShipmentUnallocatedLine(input: $input, storeId: \"store_a\") {
              ... on UpdateOutboundShipmentUnallocatedLineError {
                error {
                  __typename
                }
              }
            }
          }
        "#;

        // RecordNotFound
        let test_service = TestService(Box::new(|_| Err(ServiceError::LineDoesNotExist)));

        let expected = json!({
            "updateOutboundShipmentUnallocatedLine": {
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
    }

    #[actix_rt::test]
    async fn test_graphql_update_unallocated_standard_errors() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            InvoiceLineMutations,
            "test_graphql_update_unallocated_line_standard_errors",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: UpdateOutboundShipmentUnallocatedLineInput!) {
            updateOutboundShipmentUnallocatedLine(input: $input, storeId: \"store_a\") {
                __typename
            }
          }
        "#;

        // LineIsNotUnallocatedLine
        let test_service = TestService(Box::new(|_| Err(ServiceError::LineIsNotUnallocatedLine)));
        let expected_message = "Bad user input";
        let expected_extensions =
            json!({ "details": format!("{:#?}", ServiceError::LineIsNotUnallocatedLine) });
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &Some(empty_variables()),
            &expected_message,
            Some(expected_extensions),
            Some(service_provider(test_service, &connection_manager))
        );
    }

    #[actix_rt::test]
    async fn test_graphql_update_unallocated_line_success() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            InvoiceLineMutations,
            "test_graphql_update_unallocated_line_success",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: UpdateOutboundShipmentUnallocatedLineInput!) {
            updateOutboundShipmentUnallocatedLine(input: $input, storeId: \"store_a\") {
                ... on InvoiceLineNode {
                    id
                    invoiceId
                    itemName
                }
            }
          }
        "#;

        pub fn successfull_invoice_line() -> InvoiceLine {
            InvoiceLine {
                invoice_line_row: mock_outbound_shipment_a_invoice_lines()[0].clone(),
                invoice_row: mock_outbound_shipment_a(),
                location_row_option: None,
            }
        }

        // Success
        let test_service = TestService(Box::new(|_| Ok(successfull_invoice_line())));
        let out_line = successfull_invoice_line();
        let expected = json!({
            "updateOutboundShipmentUnallocatedLine": {
                "id": out_line.invoice_line_row.id,
                "invoiceId": out_line.invoice_line_row.invoice_id,
                "itemName": out_line.invoice_line_row.item_name
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
    }
}
