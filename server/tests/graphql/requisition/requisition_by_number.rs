mod graphql {
    use repository::{
        mock::{mock_name_a, mock_request_draft_requisition, MockDataInserts},
        schema::RequisitionRowType,
        RepositoryError, Requisition, StorageConnectionManager,
    };
    use serde_json::json;
    use server::test_utils::setup_all;
    use service::{
        requisition::RequisitionServiceTrait,
        service_provider::{ServiceContext, ServiceProvider},
    };

    use crate::graphql::assert_graphql_query;

    type GetRequisitionByNumber = dyn Fn(u32, RequisitionRowType) -> Result<Option<Requisition>, RepositoryError>
        + Sync
        + Send;

    pub struct TestService(pub Box<GetRequisitionByNumber>);

    impl RequisitionServiceTrait for TestService {
        fn get_requisition_by_number(
            &self,
            _: &ServiceContext,
            _: &str,
            requisition_number: u32,
            r#type: RequisitionRowType,
        ) -> Result<Option<Requisition>, RepositoryError> {
            self.0(requisition_number, r#type)
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

    #[actix_rt::test]
    async fn test_graphql_get_requisition_by_number() {
        let (_, _, connection_manager, settings) = setup_all(
            "test_graphql_get_requisition_by_number",
            MockDataInserts::all(),
        )
        .await;

        let query = r#"
        query {
          requisitionByNumber(requisitionNumber: 0, type: REQUEST, storeId: \"store_a\") {
            __typename
            ... on RecordNotFound {
              description
            }
            ... on RequisitionNode {
              id
              otherPartyName
            }
          }
       }
        "#;

        // Not found
        let test_service = TestService(Box::new(|_, _| Ok(None)));

        let expected = json!({
            "requisitionByNumber": {
              "__typename": "RecordNotFound"
            }
          }
        );

        assert_graphql_query!(
            &settings,
            query,
            &None,
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // Found
        let test_service = TestService(Box::new(|_, _| {
            Ok(Some(Requisition {
                requisition_row: mock_request_draft_requisition(),
                name_row: mock_name_a(),
            }))
        }));

        let expected = json!({
            "requisitionByNumber": {
                "id": mock_request_draft_requisition().id,
                "otherPartyName":  mock_name_a().name

            }
          }
        );

        assert_graphql_query!(
            &settings,
            query,
            &None,
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );
    }
}
