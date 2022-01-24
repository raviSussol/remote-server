mod graphql {
    use crate::graphql::assert_graphql_query;
    use repository::{
        get_storage_connection_manager,
        mock::{
            mock_name_linked_to_store, mock_name_not_linked_to_store, mock_store_linked_to_name,
            MockDataInserts,
        },
        test_db,
        {
            mock::{mock_name_store_joins, mock_names, mock_stores},
            schema::{NameRow, NameStoreJoinRow, StoreRow},
            NameRepository, NameStoreJoinRepository, StoreRowRepository,
        },
    };
    use serde_json::json;
    use server::test_utils::{get_test_settings, setup_all};

    #[actix_rt::test]
    async fn test_graphql_names_query() {
        let settings = get_test_settings("omsupply-database-gql-names-query");
        test_db::setup(&settings.database).await;
        let connection_manager = get_storage_connection_manager(&settings.database);
        let connection = connection_manager.connection().unwrap();

        // setup
        let name_repository = NameRepository::new(&connection);
        let store_repository = StoreRowRepository::new(&connection);
        let name_store_repository = NameStoreJoinRepository::new(&connection);
        let mut mock_names: Vec<NameRow> = mock_names();
        mock_names.sort_by(|a, b| a.id.cmp(&b.id));

        let mock_stores: Vec<StoreRow> = mock_stores();
        let mock_name_store_joins: Vec<NameStoreJoinRow> = mock_name_store_joins();
        for name in &mock_names {
            name_repository.insert_one(&name).await.unwrap();
        }
        for store in &mock_stores {
            store_repository.insert_one(&store).await.unwrap();
        }
        for name_store_join in &mock_name_store_joins {
            name_store_repository.upsert_one(name_store_join).unwrap();
        }

        let query = r#"{
            names {
                ... on NameConnector {
                  nodes{
                      id
                  }
               }
            }
        }"#;
        let expected = json!({
          "names": {
              "nodes": mock_names.iter().map(|name| json!({
                "id": name.id,
              })).collect::<serde_json::Value>(),
            }
          }
        );
        assert_graphql_query!(&settings, query, &None, &expected, None);

        // test sorting
        let query = r#"query Names($sort: [NameSortInput]) {
          names(sort: $sort){
              ... on NameConnector {
                nodes {
                    name
                }
              }
          }
        }"#;
        let variables = Some(json!({
          "sort": [{
            "key": "name",
            "desc": true,
          }]
        }));
        let mut sorted_mock_names = mock_names.clone();
        sorted_mock_names.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()));
        let expected = json!({
          "names": {
              "nodes": sorted_mock_names.iter().map(|name| json!({
                "name": name.name,
              })).collect::<serde_json::Value>(),
            }
          }
        );
        assert_graphql_query!(&settings, query, &variables, &expected, None);

        // test filtering
        let query = r#"query Names($filter: [NameFilterInput]) {
          names(filter: $filter){
              ... on NameConnector {
                nodes {
                    id
                }
              }
          }
        }"#;
        let variables = Some(json!({
          "filter": {
            "isCustomer": true,
          }
        }));
        let expected_names_ids: Vec<&String> = mock_name_store_joins
            .iter()
            .filter(|a| a.name_is_customer)
            .map(|a| &a.name_id)
            .collect();
        let names: Vec<&NameRow> = mock_names
            .iter()
            .filter(|a| {
                expected_names_ids
                    .iter()
                    .find(|search_id| search_id == &&&a.id)
                    .is_some()
            })
            .collect();
        let expected = json!({
          "names": {
              "nodes": names.iter().map(|name| json!({
                "id": name.id,
              })).collect::<serde_json::Value>(),
            }
          }
        );
        assert_graphql_query!(&settings, query, &variables, &expected, None);
    }

    #[actix_rt::test]
    async fn test_graphql_names_query_loaders() {
        let (_, _, _, settings) =
            setup_all("test_graphql_names_query_loaders", MockDataInserts::all()).await;

        let query = r#"query Names($filter: NameFilterInput!) {
              names(filter: $filter){
                  ... on NameConnector {
                    nodes {
                        store {
                          id
                        }
                    }
                  }
              }
            }"#;

        // Test store loader, name linked to store
        let variables = Some(json!({
          "filter": {
            "id": { "equalTo": mock_name_linked_to_store().id }
          }
        }));

        let expected = json!({
          "names": {
              "nodes": [
               {
                "store": {
                  "id": mock_store_linked_to_name().id,
                }
               }
              ]
            }
          }
        );
        assert_graphql_query!(&settings, query, &variables, &expected, None);

        // Test store loader, name not linked to store
        let variables = Some(json!({
          "filter": {
            "id": { "equalTo": mock_name_not_linked_to_store().id }
          }
        }));

        let expected = json!({
          "names": {
              "nodes": [
               {
                "store": null
               }
              ]
            }
          }
        );
        assert_graphql_query!(&settings, query, &variables, &expected, None);
    }
}
