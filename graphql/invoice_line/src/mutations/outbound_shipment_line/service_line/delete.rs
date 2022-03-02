use async_graphql::*;
use graphql_core::ContextExt;
use service::invoice_line::outbound_shipment_service_line::DeleteOutboundShipmentServiceLineError as ServiceError;

use graphql_core::simple_generic_errors::{
    CannotEditInvoice, ForeignKey, ForeignKeyError, RecordNotFound,
};
use graphql_core::standard_graphql_error::StandardGraphqlError;
use graphql_types::types::DeleteResponse as GenericDeleteResponse;

use service::invoice_line::outbound_shipment_line::DeleteOutboundShipmentLine as ServiceInput;

#[derive(InputObject)]
#[graphql(name = "DeleteOutboundShipmentServiceLineInput")]
pub struct DeleteInput {
    pub id: String,
    pub invoice_id: String,
}

#[derive(SimpleObject)]
#[graphql(name = "DeleteOutboundShipmentServiceLineError")]
pub struct DeleteError {
    pub error: DeleteErrorInterface,
}

#[derive(Union)]
#[graphql(name = "DeleteOutboundShipmentServiceLineResponse")]
pub enum DeleteResponse {
    Error(DeleteError),
    Response(GenericDeleteResponse),
}

pub fn delete(ctx: &Context<'_>, store_id: &str, input: DeleteInput) -> Result<DeleteResponse> {
    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;

    map_response(
        service_provider
            .invoice_line_service
            .delete_outbound_shipment_service_line(&service_context, store_id, input.to_domain()),
    )
}

pub fn map_response(from: Result<String, ServiceError>) -> Result<DeleteResponse> {
    let result = match from {
        Ok(id) => DeleteResponse::Response(GenericDeleteResponse(id)),
        Err(error) => DeleteResponse::Error(DeleteError {
            error: map_error(error)?,
        }),
    };

    Ok(result)
}

#[derive(Interface)]
#[graphql(name = "DeleteOutboundShipmentServiceLineErrorInterface")]
#[graphql(field(name = "description", type = "&str"))]
pub enum DeleteErrorInterface {
    RecordNotFound(RecordNotFound),
    ForeignKeyError(ForeignKeyError),
    CannotEditInvoice(CannotEditInvoice),
}

impl DeleteInput {
    pub fn to_domain(self) -> ServiceInput {
        let DeleteInput { id, invoice_id } = self;
        ServiceInput { id, invoice_id }
    }
}

fn map_error(error: ServiceError) -> Result<DeleteErrorInterface> {
    use StandardGraphqlError::*;
    let formatted_error = format!("{:#?}", error);

    let graphql_error = match error {
        // Structured Errors
        ServiceError::LineDoesNotExist => {
            return Ok(DeleteErrorInterface::RecordNotFound(RecordNotFound {}))
        }
        ServiceError::CannotEditInvoice => {
            return Ok(DeleteErrorInterface::CannotEditInvoice(
                CannotEditInvoice {},
            ))
        }
        ServiceError::InvoiceDoesNotExist => {
            return Ok(DeleteErrorInterface::ForeignKeyError(ForeignKeyError(
                ForeignKey::InvoiceId,
            )))
        }
        // Standard Graphql Errors
        ServiceError::NotThisInvoiceLine(_) => BadUserInput(formatted_error),
        ServiceError::NotAnOutboundShipment => BadUserInput(formatted_error),
        ServiceError::DatabaseError(_) => InternalError(formatted_error),
    };

    Err(graphql_error.extend())
}

#[cfg(test)]
mod test {
    use crate::InvoiceLineMutations;
    use async_graphql::EmptyMutation;
    use graphql_core::{
        assert_graphql_query, assert_standard_graphql_error, test_helpers::setup_graphl_test,
    };

    use repository::{mock::MockDataInserts, StorageConnectionManager};
    use serde_json::json;
    use service::{
        invoice_line::{
            outbound_shipment_line::DeleteOutboundShipmentLine,
            outbound_shipment_service_line::DeleteOutboundShipmentServiceLineError,
            InvoiceLineServiceTrait,
        },
        service_provider::{ServiceContext, ServiceProvider},
    };

    type ServiceInput = DeleteOutboundShipmentLine;
    type ServiceError = DeleteOutboundShipmentServiceLineError;

    type DeleteLineMethod =
        dyn Fn(&str, ServiceInput) -> Result<String, ServiceError> + Sync + Send;

    pub struct TestService(pub Box<DeleteLineMethod>);

    impl InvoiceLineServiceTrait for TestService {
        fn delete_outbound_shipment_service_line(
            &self,
            _: &ServiceContext,
            store_id: &str,
            input: ServiceInput,
        ) -> Result<String, ServiceError> {
            self.0(store_id, input)
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

    #[actix_rt::test]
    async fn test_graphql_delete_outbound_shipment_service_line() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            InvoiceLineMutations,
            "test_graphql_delete_outbound_shipment_service_line",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: DeleteOutboundShipmentServiceLineInput!, $storeId: String) {
            deleteOutboundShipmentServiceLine(storeId: $storeId, input: $input) {
              ... on DeleteOutboundShipmentServiceLineError {
                error {
                  __typename
                }
              }
            }
          }
        "#;

        let variables = Some(json!({
            "storeId": "store_a",
            "input": {
                "id": "n/a",
                "invoiceId": "n/a"
            }
        }));

        // LineDoesNotExist
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::LineDoesNotExist)));

        let expected = json!({
            "deleteOutboundShipmentServiceLine": {
              "error": {
                "__typename": "RecordNotFound"
              }
            }
          }
        );

        assert_graphql_query!(
            &settings,
            mutation,
            &variables,
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // InvoiceDoesNotExist
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::InvoiceDoesNotExist)));

        let expected = json!({
            "deleteOutboundShipmentServiceLine": {
              "error": {
                "__typename": "ForeignKeyError"
              }
            }
          }
        );

        assert_graphql_query!(
            &settings,
            mutation,
            &variables,
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // CannotEditInvoice
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::CannotEditInvoice)));

        let expected = json!({
            "deleteOutboundShipmentServiceLine": {
              "error": {
                "__typename": "CannotEditInvoice"
              }
            }
          }
        );

        assert_graphql_query!(
            &settings,
            mutation,
            &variables,
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // NotAnOutboundShipment
        let test_service = TestService(Box::new(|_, _| Err(ServiceError::NotAnOutboundShipment)));
        let expected_message = "Bad user input";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &variables,
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        // NotThisInvoiceLine
        let test_service = TestService(Box::new(|_, _| {
            Err(ServiceError::NotThisInvoiceLine("id".to_string()))
        }));
        let expected_message = "Bad user input";
        assert_standard_graphql_error!(
            &settings,
            &mutation,
            &variables,
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );
    }

    #[actix_rt::test]
    async fn test_graphql_delete_request_invoice_line_success() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            InvoiceLineMutations,
            "test_graphql_delete_request_invoice_line_success",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: DeleteOutboundShipmentServiceLineInput!, $storeId: String) {
            deleteOutboundShipmentServiceLine(storeId: $storeId, input: $input) {
              ... on DeleteResponse {
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
                    id: "delete line id input".to_string(),
                    invoice_id: "invoice_id".to_string(),
                }
            );
            Ok("delete line id input".to_string())
        }));

        let variables = json!({
          "input": {
            "id": "delete line id input",
            "invoiceId": "invoice_id",
          },
          "storeId": "store_a"
        });

        let expected = json!({
            "deleteOutboundShipmentServiceLine": {
                "id":  "delete line id input".to_string()
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
