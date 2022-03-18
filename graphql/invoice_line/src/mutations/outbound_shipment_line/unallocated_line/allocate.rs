use async_graphql::*;
use graphql_core::{
    simple_generic_errors::RecordNotFound, standard_graphql_error::validate_auth,
    standard_graphql_error::StandardGraphqlError, ContextExt,
};
use graphql_types::types::{DeleteResponse, InvoiceLineConnector};
use service::{
    invoice_line::outbound_shipment_unallocated_line::{
        AllocateOutboundShipmentUnallocatedLineError as ServiceError,
        InvoiceLineInsertsUpdatesDeletes,
    },
    permission_validation::{Resource, ResourceAccessRequest},
};

#[derive(Interface)]
#[graphql(name = "AllocateOutboundShipmentUnallocatedLineErrorInterface")]
#[graphql(field(name = "description", type = "String"))]
pub enum AllocateErrorInterface {
    RecordNotFound(RecordNotFound),
}

#[derive(SimpleObject)]
#[graphql(name = "AllocateOutboundShipmentUnallocatedLineError")]
pub struct AllocateError {
    pub error: AllocateErrorInterface,
}

#[derive(Union)]
#[graphql(name = "AllocateOutboundShipmentUnallocatedLineResponse")]
pub enum AllocateResponse {
    Error(AllocateError),
    Response(ResponseNode),
}
#[derive(SimpleObject)]
#[graphql(name = "AllocateOutboundShipmentUnallocatedLineNode")]
pub struct ResponseNode {
    updates: InvoiceLineConnector,
    inserts: InvoiceLineConnector,
    deletes: Vec<DeleteResponse>,
}

pub fn allocate(ctx: &Context<'_>, store_id: &str, line_id: String) -> Result<AllocateResponse> {
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
            .allocate_outbound_shipment_unallocated_line(&service_context, store_id, line_id),
    )
}

pub fn map_response(
    from: Result<InvoiceLineInsertsUpdatesDeletes, ServiceError>,
) -> Result<AllocateResponse> {
    let result = match from {
        Ok(line) => AllocateResponse::Response(ResponseNode::from_domain(line)),
        Err(error) => AllocateResponse::Error(AllocateError {
            error: map_error(error)?,
        }),
    };

    Ok(result)
}

fn map_error(error: ServiceError) -> Result<AllocateErrorInterface> {
    use StandardGraphqlError::*;
    let formatted_error = format!("{:#?}", error);

    let graphql_error = match error {
        // Structured Errors
        ServiceError::LineDoesNotExist => {
            return Ok(AllocateErrorInterface::RecordNotFound(RecordNotFound {}))
        }
        // Standard Graphql Errors
        ServiceError::LineIsNotUnallocatedLine => BadUserInput(formatted_error),
        ServiceError::InsertOutboundShipmentLine(_) => InternalError(formatted_error),
        ServiceError::UpdateOutboundShipmentLine(_) => InternalError(formatted_error),
        ServiceError::DeleteOutboundShipmentUnallocatedLine(_) => InternalError(formatted_error),
        ServiceError::UpdateOutboundShipmentUnallocatedLine(_) => InternalError(formatted_error),
        ServiceError::DatabaseError(_) => InternalError(formatted_error),
    };

    Err(graphql_error.extend())
}

impl ResponseNode {
    pub fn from_domain(from: InvoiceLineInsertsUpdatesDeletes) -> ResponseNode {
        let InvoiceLineInsertsUpdatesDeletes {
            updates,
            deletes,
            inserts,
        } = from;
        ResponseNode {
            updates: InvoiceLineConnector::from_vec(updates),
            deletes: deletes.into_iter().map(|id| DeleteResponse(id)).collect(),
            inserts: InvoiceLineConnector::from_vec(inserts),
        }
    }
}

#[cfg(test)]
mod graphql {
    use async_graphql::EmptyMutation;
    use graphql_core::{
        assert_graphql_query, assert_standard_graphql_error, test_helpers::setup_graphl_test,
    };
    use repository::{
        mock::MockDataInserts, schema::InvoiceLineRow, InvoiceLine, StorageConnectionManager,
    };
    use serde_json::json;

    use service::{
        invoice_line::{
            outbound_shipment_unallocated_line::{
                AllocateOutboundShipmentUnallocatedLineError as ServiceError,
                InvoiceLineInsertsUpdatesDeletes,
            },
            InvoiceLineServiceTrait,
        },
        service_provider::{ServiceContext, ServiceProvider},
    };
    use util::inline_init;

    use crate::InvoiceLineMutations;

    type AllocateLineMethod =
        dyn Fn(String) -> Result<InvoiceLineInsertsUpdatesDeletes, ServiceError> + Sync + Send;

    pub struct TestService(pub Box<AllocateLineMethod>);

    impl InvoiceLineServiceTrait for TestService {
        fn allocate_outbound_shipment_unallocated_line(
            &self,
            _: &ServiceContext,
            _: &str,
            input: String,
        ) -> Result<InvoiceLineInsertsUpdatesDeletes, ServiceError> {
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
          "lineId": "unallocated_line"
        })
    }

    #[actix_rt::test]
    async fn test_graphql_allocate_unallocated_structured_errors() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            InvoiceLineMutations,
            "test_graphql_allocate_unallocated_line_structured_errors",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($lineId: String!) {
            allocateOutboundShipmentUnallocatedLine(lineId: $lineId, storeId: \"store_a\") {
              ... on AllocateOutboundShipmentUnallocatedLineError {
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
            "allocateOutboundShipmentUnallocatedLine": {
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
    async fn test_graphql_allocate_unallocated_standard_errors() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            InvoiceLineMutations,
            "test_graphql_allocate_unallocated_line_standard_errors",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($lineId: String!) {
            allocateOutboundShipmentUnallocatedLine(lineId: $lineId, storeId: \"store_a\") {
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
    async fn test_graphql_allocate_unallocated_line_success() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            InvoiceLineMutations,
            "test_graphql_allocate_unallocated_line_success",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($lineId: String!) {
            allocateOutboundShipmentUnallocatedLine(lineId: $lineId, storeId: \"store_a\") {
                ... on AllocateOutboundShipmentUnallocatedLineNode {
                    inserts {
                        nodes {
                            id
                        }
                    }
                    updates {
                        nodes {
                            id
                        }
                    } 
                    deletes {
                        id
                    }
                }
            }
          }
        "#;

        // Success
        let test_service = TestService(Box::new(|line_id| {
            assert_eq!(line_id, "unallocated_line");
            Ok(InvoiceLineInsertsUpdatesDeletes {
                inserts: vec![inline_init(|r: &mut InvoiceLine| {
                    r.invoice_line_row =
                        inline_init(|r: &mut InvoiceLineRow| r.id = "insert1".to_string())
                })],
                deletes: vec!["delete1".to_string()],
                updates: vec![
                    inline_init(|r: &mut InvoiceLine| {
                        r.invoice_line_row =
                            inline_init(|r: &mut InvoiceLineRow| r.id = "update1".to_string())
                    }),
                    inline_init(|r: &mut InvoiceLine| {
                        r.invoice_line_row =
                            inline_init(|r: &mut InvoiceLineRow| r.id = "update2".to_string())
                    }),
                ],
            })
        }));

        let expected = json!({
            "allocateOutboundShipmentUnallocatedLine": {
                "inserts": {
                    "nodes": [{
                        "id": "insert1"
                    }]
                },
                "deletes": [ {
                    "id": "delete1"
                }],
                "updates": {
                    "nodes": [{
                        "id": "update1"
                    },{
                        "id": "update2"
                    }]
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
}
