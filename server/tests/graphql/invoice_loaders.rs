mod graphql {
    use crate::graphql::assert_graphql_query;
    use repository::mock::{
        mock_invoice_loader_invoice1, mock_invoice_loader_invoice2,
        mock_invoice_loader_requistion1, MockDataInserts,
    };
    use serde_json::json;
    use server::test_utils::setup_all;

    #[actix_rt::test]
    async fn test_graphql_invoice_loaders() {
        let (_, _, _, settings) =
            setup_all("test_graphql_invoice_loaders", MockDataInserts::all()).await;

        let query = r#"
        query($filter: InvoiceFilterInput) {
          invoices(filter: $filter, storeId: \"store_a\") {
            ... on InvoiceConnector {
              nodes {
                id
                linkedShipment {
                    id
                }
                requisition {
                    id
                }
              }
            }
          }
       }
        "#;

        let invoice1 = mock_invoice_loader_invoice1();
        let invoice2 = mock_invoice_loader_invoice2();

        let variables = json!({
          "filter": {
            "id": {
                "equalAny": [&invoice1.id, &invoice2.id]
            },
          }
        }
        );

        let expected = json!({
            "invoices": {
                "nodes": [{
                    "id": &invoice1.id,
                    "linkedShipment": null,
                    "requisition": {
                        "id": mock_invoice_loader_requistion1().id
                    }
                },
                {
                    "id": &invoice2.id,
                     "linkedShipment": {
                        "id": &invoice1.id
                    },
                    "requisition": null
                }]
            }
        }
        );

        assert_graphql_query!(&settings, query, &Some(variables.clone()), &expected, None);
    }
}
