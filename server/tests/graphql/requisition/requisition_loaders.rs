mod graphql {

    use repository::mock::{
        mock_invoice1_linked_to_requisition, mock_invoice2_linked_to_requisition,
        mock_invoice3_linked_to_requisition, mock_name_a, mock_name_store_a,
        mock_request_draft_requisition_all_fields, mock_response_draft_requisition_all_fields,
        MockDataInserts,
    };
    use serde_json::json;
    use server::test_utils::setup_all;

    use crate::graphql::assert_graphql_query;

    #[actix_rt::test]
    async fn test_graphql_requisition_loaders() {
        let (_, _, _, settings) =
            setup_all("test_graphql_requisition_loaders", MockDataInserts::all()).await;

        let query = r#"
        query($filter: RequisitionFilterInput) {
          requisitions(filter: $filter, storeId: \"store_a\") {
            ... on RequisitionConnector {
              nodes {
                id
                otherParty {
                    id
                }
                requestRequisition {
                    id
                }
                shipments {
                    nodes {
                        id
                    }
                    totalCount
                }
              }
            }
          }
       }
        "#;

        let request_requisition = mock_request_draft_requisition_all_fields();
        let response_requisition = mock_response_draft_requisition_all_fields();

        let variables = json!({
          "filter": {
            "id": {
                "equalAny": [&request_requisition.requisition.id, &response_requisition.requisition.id]
            },
          }
        }
        );

        // Test otherParty

        let expected = json!({
            "requisitions": {
                "nodes": [{
                    "id": &request_requisition.requisition.id,
                    "otherParty": {
                        "id": mock_name_a().id
                    }
                },
                {
                    "id": &response_requisition.requisition.id,
                    "otherParty": {
                        "id": mock_name_store_a().id
                    },
                }]
            }
        }
        );

        assert_graphql_query!(&settings, query, &Some(variables.clone()), &expected, None);

        // Test requestRequisition
        let expected = json!({
            "requisitions": {
                "nodes": [{
                    "id": &request_requisition.requisition.id,
                    "requestRequisition": null,
                },
                {
                    "id": &response_requisition.requisition.id,
                    "requestRequisition": {
                        "id": &request_requisition.requisition.id
                    },
                }]
            }
        }
        );

        assert_graphql_query!(&settings, query, &Some(variables.clone()), &expected, None);

        // Test shippents
        let expected = json!({
            "requisitions": {
                "nodes": [{
                    "id": &request_requisition.requisition.id,
                    "shipments": {
                        "nodes": [{
                            "id": mock_invoice1_linked_to_requisition().invoice.id,
                        },
                        {
                            "id": mock_invoice2_linked_to_requisition().invoice.id,
                        }],
                        "totalCount": 2
                    },
                },
                {
                    "id": &response_requisition.requisition.id,
                    "shipments": {
                        "nodes": [{
                            "id": mock_invoice3_linked_to_requisition().invoice.id,
                        }],
                        "totalCount": 1
                    },
                }]
            }
        }
        );

        assert_graphql_query!(&settings, query, &Some(variables.clone()), &expected, None);
    }

    #[actix_rt::test]
    async fn test_graphql_requisition_line() {
        let (_, _, _, settings) =
            setup_all("test_graphql_requisition_line", MockDataInserts::all()).await;

        let query = r#"
        query($filter: RequisitionFilterInput) {
          requisitions(filter: $filter, storeId: \"store_a\") {
            ... on RequisitionConnector {
              nodes {
                id
                lines {
                    totalCount
                    nodes {
                        id
                        itemId
                        requestedQuantity
                        supplyQuantity
                        calculatedQuantity
                        itemStats {
                            averageMonthlyConsumption
                            stockOnHand
                            monthsOfStock
                        }
                    } 
                }
              }
            }
          }
       }
        "#;

        let response_requisition = mock_response_draft_requisition_all_fields();

        let variables = json!({
          "filter": {
                "id": {
                    "equalTo": &response_requisition.requisition.id,
                },
            }
        }
        );

        // Test item

        let expected = json!({
            "requisitions": {
                "nodes": [{
                    "id": &response_requisition.requisition.id,
                    "lines": {
                        "totalCount": 1,
                         "nodes": [{
                            "id": &response_requisition.lines[0].id,
                            "itemId":&response_requisition.lines[0].item_id,
                            "requestedQuantity": &response_requisition.lines[0].requested_quantity,
                            "supplyQuantity":&response_requisition.lines[0].supply_quantity,
                            "calculatedQuantity":&response_requisition.lines[0].calculated_quantity,
                            "itemStats": {
                                "averageMonthlyConsumption": &response_requisition.lines[0].average_monthly_consumption,
                                "stockOnHand": &response_requisition.lines[0].stock_on_hand,
                                "monthsOfStock": response_requisition.lines[0].stock_on_hand as f64 / response_requisition.lines[0].average_monthly_consumption as f64
                            }
                         }]
                    }
                }]
            }
        }
        );

        assert_graphql_query!(&settings, query, &Some(variables.clone()), &expected, None);
    }

    #[actix_rt::test]
    async fn test_graphql_requisition_line_loaders() {
        let (_, _, _, settings) = setup_all(
            "test_graphql_requisition_line_loaders",
            MockDataInserts::all(),
        )
        .await;

        let query = r#"
        query($filter: RequisitionFilterInput) {
          requisitions(filter: $filter, storeId: \"store_a\") {
            ... on RequisitionConnector {
              nodes {
                id
                lines {
                    nodes {
                        id
                        item {
                            id
                        }
                        linkedRequisitionLine {
                            id
                        }
                    }
                }
              }
            }
          }
       }
        "#;

        let request_requisition = mock_request_draft_requisition_all_fields();
        let response_requisition = mock_response_draft_requisition_all_fields();

        let variables = json!({
          "filter": {
            "id": {
                "equalAny": [&request_requisition.requisition.id, &response_requisition.requisition.id]
            },
          }
        }
        );

        // Test item and linked requisition line

        let expected = json!({
            "requisitions": {
                "nodes": [{
                    "id": &request_requisition.requisition.id,
                    "lines": {
                         "nodes": [{
                             "item": {
                                 "id": request_requisition.lines[0].item_id
                             },
                             "linkedRequisitionLine": {
                                "id": response_requisition.lines[0].id,
                             }
                         },{
                            "item": {
                                "id": request_requisition.lines[1].item_id
                            },
                            "linkedRequisitionLine": null
                        }]
                    }
                },{
                    "id": &response_requisition.requisition.id,
                    "lines": {
                         "nodes": [{
                             "item": {
                                 "id": response_requisition.lines[0].item_id
                             },
                             "linkedRequisitionLine": {
                                "id": request_requisition.lines[0].id,
                            }
                         }]
                    }
                }]
            }
        }
        );

        assert_graphql_query!(&settings, query, &Some(variables.clone()), &expected, None);

        // Test inbound/outbound shipment lines

        let query = r#"
        query($filter: RequisitionFilterInput) {
          requisitions(filter: $filter, storeId: \"store_a\") {
            ... on RequisitionConnector {
              nodes {
                id
                lines {
                    nodes {
                        outboundShipmentLines {
                            nodes {
                                id
                            }
                            totalCount
                        } 
                        inboundShipmentLines {
                            nodes {
                                id
                            }
                            totalCount
                        } 
                    }
                }
              }
            }
          }
       }
        "#;

        let expected = json!({
            "requisitions": {
                "nodes": [{
                    "id": &request_requisition.requisition.id,
                    "lines": {
                         "nodes": [{
                             "outboundShipmentLines": {
                                 "nodes": [{
                                    "id": mock_invoice3_linked_to_requisition().lines[0].line.id,
                                 }],
                             },
                             "inboundShipmentLines": {
                                "nodes": [{
                                   "id": mock_invoice1_linked_to_requisition().lines[0].line.id,
                                }],
                            }
                         },{
                            "outboundShipmentLines": {
                                "totalCount": 0
                            },
                            "inboundShipmentLines": {
                                "totalCount": 2
                            }
                        }]
                    }
                },{
                    "id": &response_requisition.requisition.id,
                    "lines": {
                         "nodes": [{
                            "outboundShipmentLines": {
                                "nodes": [{
                                   "id": mock_invoice3_linked_to_requisition().lines[0].line.id,
                                }],
                            },
                            "inboundShipmentLines": {
                               "nodes": [{
                                  "id": mock_invoice1_linked_to_requisition().lines[0].line.id,
                               }],
                           }
                         }]
                    }
                }]
            }
        }
        );

        assert_graphql_query!(&settings, query, &Some(variables.clone()), &expected, None);
    }
}
