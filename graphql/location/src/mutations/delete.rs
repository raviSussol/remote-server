use async_graphql::*;
use graphql_core::{
    simple_generic_errors::{DatabaseError, RecordBelongsToAnotherStore, RecordNotFound},
    standard_graphql_error::validate_auth,
    ContextExt,
};
use graphql_types::types::{DeleteResponse, InvoiceLineConnector, StockLineConnector};
use repository::RepositoryError;
use service::{
    location::delete::{
        DeleteLocation, DeleteLocationError as InError, LocationInUse as ServiceLocationInUse,
    },
    permission_validation::{Resource, ResourceAccessRequest},
};

pub fn delete_location(
    ctx: &Context<'_>,
    store_id: &str,
    input: DeleteLocationInput,
) -> Result<DeleteLocationResponse> {
    validate_auth(
        ctx,
        &ResourceAccessRequest {
            resource: Resource::MutateLocation,
            store_id: Some(store_id.to_string()),
        },
    )?;

    let service_provider = ctx.service_provider();
    let service_context = match service_provider.context() {
        Ok(service) => service,
        Err(error) => return Ok(DeleteLocationResponse::Error(error.into())),
    };

    match service_provider.location_service.delete_location(
        &service_context,
        store_id,
        input.into(),
    ) {
        Ok(location_id) => Ok(DeleteLocationResponse::Response(DeleteResponse(
            location_id,
        ))),
        Err(error) => Ok(DeleteLocationResponse::Error(error.into())),
    }
}

#[derive(InputObject)]
pub struct DeleteLocationInput {
    pub id: String,
}

impl From<DeleteLocationInput> for DeleteLocation {
    fn from(DeleteLocationInput { id }: DeleteLocationInput) -> Self {
        DeleteLocation { id }
    }
}

#[derive(SimpleObject)]
pub struct DeleteLocationError {
    pub error: DeleteLocationErrorInterface,
}

#[derive(Union)]
pub enum DeleteLocationResponse {
    Error(DeleteLocationError),
    Response(DeleteResponse),
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "String"))]
pub enum DeleteLocationErrorInterface {
    LocationNotFound(RecordNotFound),
    RecordBelongsToAnotherStore(RecordBelongsToAnotherStore),
    LocationInUse(LocationInUse),
    DatabaseError(DatabaseError),
}

pub struct LocationInUse {
    stock_lines: StockLineConnector,
    invoice_lines: InvoiceLineConnector,
}

#[Object]
impl LocationInUse {
    pub async fn description(&self) -> &'static str {
        "Location in use"
    }

    pub async fn stock_lines(&self) -> &StockLineConnector {
        &self.stock_lines
    }

    pub async fn invoice_lines(&self) -> &InvoiceLineConnector {
        &self.invoice_lines
    }
}

impl From<RepositoryError> for DeleteLocationError {
    fn from(error: RepositoryError) -> Self {
        let error = DeleteLocationErrorInterface::DatabaseError(DatabaseError(error));
        DeleteLocationError { error }
    }
}

impl From<InError> for DeleteLocationError {
    fn from(error: InError) -> Self {
        use DeleteLocationErrorInterface as OutError;
        let error = match error {
            InError::LocationDoesNotExist => OutError::LocationNotFound(RecordNotFound {}),
            InError::LocationInUse(ServiceLocationInUse {
                stock_lines,
                invoice_lines,
            }) => OutError::LocationInUse(LocationInUse {
                stock_lines: StockLineConnector::from_vec(stock_lines),
                invoice_lines: InvoiceLineConnector::from_vec(invoice_lines),
            }),
            InError::LocationDoesNotBelongToCurrentStore => {
                OutError::RecordBelongsToAnotherStore(RecordBelongsToAnotherStore {})
            }
            InError::DatabaseError(error) => OutError::DatabaseError(DatabaseError(error)),
        };
        DeleteLocationError { error }
    }
}

#[cfg(test)]
mod test {
    use async_graphql::EmptyMutation;
    use graphql_core::{assert_graphql_query, test_helpers::setup_graphl_test};
    use repository::{
        mock::{
            mock_outbound_shipment_a, mock_outbound_shipment_a_invoice_lines, mock_stock_line_a,
            MockDataInserts,
        },
        InvoiceLine, StockLine, StorageConnectionManager,
    };
    use serde_json::json;

    use service::{
        location::{
            delete::{DeleteLocation, DeleteLocationError, LocationInUse},
            LocationServiceTrait,
        },
        service_provider::{ServiceContext, ServiceProvider},
    };

    use crate::LocationMutations;

    type DeleteLocationMethod =
        dyn Fn(DeleteLocation) -> Result<String, DeleteLocationError> + Sync + Send;

    pub struct TestService(pub Box<DeleteLocationMethod>);

    impl LocationServiceTrait for TestService {
        fn delete_location(
            &self,
            _: &ServiceContext,
            _: &str,
            input: DeleteLocation,
        ) -> Result<String, DeleteLocationError> {
            (self.0)(input)
        }
    }

    pub fn service_provider(
        location_service: TestService,
        connection_manager: &StorageConnectionManager,
    ) -> ServiceProvider {
        let mut service_provider = ServiceProvider::new(connection_manager.clone());
        service_provider.location_service = Box::new(location_service);
        service_provider
    }

    #[actix_rt::test]
    async fn test_graphql_delete_location_errors() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            LocationMutations,
            "test_graphql_delete_location_errors",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: DeleteLocationInput!) {
            deleteLocation(input: $input, storeId: \"store_a\") {
              ... on DeleteLocationError {
                error {
                  __typename
                }
              }
            }
          }
        "#;

        let variables = Some(json!({
          "input": {
            "id": "n/a",
          }
        }));

        // Record Not Found
        let test_service =
            TestService(Box::new(|_| Err(DeleteLocationError::LocationDoesNotExist)));

        let expected = json!({
            "deleteLocation": {
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

        // Not current store location
        let test_service = TestService(Box::new(|_| {
            Err(DeleteLocationError::LocationDoesNotBelongToCurrentStore)
        }));

        let expected = json!({
            "deleteLocation": {
              "error": {
                "__typename": "RecordBelongsToAnotherStore",
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

        // Location in use
        let mutation = r#"
        mutation ($input: DeleteLocationInput!) {
            deleteLocation(input: $input, storeId: \"store_a\") {
              ... on DeleteLocationError {
                error {
                  __typename
                  ... on LocationInUse {
                    stockLines {
                      nodes {
                        id
                      }
                    }
                    invoiceLines {
                      nodes {
                        id
                      }
                    }
                  }
                }
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

        let test_service = TestService(Box::new(|_| {
            Err(DeleteLocationError::LocationInUse(LocationInUse {
                stock_lines: vec![StockLine {
                    stock_line_row: mock_stock_line_a(),
                    location_row: None,
                }],
                invoice_lines: vec![successfull_invoice_line()],
            }))
        }));

        // let invoice_line_ids = stock_lines.iter();
        let out_line = successfull_invoice_line();
        let expected = json!({
            "deleteLocation": {
              "error": {
                "__typename": "LocationInUse",
                "stockLines": {
                  "nodes": [{"id": mock_stock_line_a().id}]
                },
                "invoiceLines": {
                  "nodes": [{"id": out_line.invoice_line_row.id}]
                }
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
    }

    #[actix_rt::test]
    async fn test_graphql_delete_location_success() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            EmptyMutation,
            LocationMutations,
            "test_graphql_delete_location_success",
            MockDataInserts::all(),
        )
        .await;

        let mutation = r#"
        mutation ($input: DeleteLocationInput!) {
            deleteLocation(input: $input, storeId: \"store_a\") {
              ... on DeleteResponse {
                id
              }
            }
          }
        "#;

        let variables = Some(json!({
          "input": {
            "id": "n/a",

          }
        }));

        let test_service = TestService(Box::new(|_| Ok("deleted".to_owned())));

        let expected = json!({
            "deleteLocation": {
                "id": "deleted",
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
    }
}
