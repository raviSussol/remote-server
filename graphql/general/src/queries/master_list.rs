use async_graphql::*;
use graphql_core::{
    generic_filters::{EqualFilterStringInput, SimpleStringFilterInput},
    pagination::PaginationInput,
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};
use graphql_types::types::MasterListNode;
use repository::{EqualFilter, PaginationOption, SimpleStringFilter};
use repository::{MasterList, MasterListFilter, MasterListSort};
use service::ListResult;
#[derive(Enum, Copy, Clone, PartialEq, Eq)]
#[graphql(remote = "repository::MasterListSortField")]
#[graphql(rename_items = "camelCase")]
pub enum MasterListSortFieldInput {
    Name,
    Code,
    Description,
}

#[derive(InputObject)]
pub struct MasterListSortInput {
    /// Sort query result by `key`
    key: MasterListSortFieldInput,
    /// Sort query result is sorted descending or ascending (if not provided the default is
    /// ascending)
    desc: Option<bool>,
}

impl MasterListSortInput {
    pub fn to_domain(self) -> MasterListSort {
        MasterListSort {
            // From trait is auto implemented by graphql(remote) in MasterListSortFieldInput
            key: self.key.into(),
            desc: self.desc,
        }
    }
}

#[derive(InputObject, Clone)]
pub struct MasterListFilterInput {
    pub id: Option<EqualFilterStringInput>,
    pub name: Option<SimpleStringFilterInput>,
    pub code: Option<SimpleStringFilterInput>,
    pub description: Option<SimpleStringFilterInput>,
    pub exists_for_name: Option<SimpleStringFilterInput>,
    pub exists_for_name_id: Option<EqualFilterStringInput>,
    pub exists_for_store_id: Option<EqualFilterStringInput>,
}

impl MasterListFilterInput {
    pub fn to_domain(self) -> MasterListFilter {
        MasterListFilter {
            id: self.id.map(EqualFilter::from),
            name: self.name.map(SimpleStringFilter::from),
            code: self.code.map(SimpleStringFilter::from),
            description: self.description.map(SimpleStringFilter::from),
            exists_for_name: self.exists_for_name.map(SimpleStringFilter::from),
            exists_for_name_id: self.exists_for_name_id.map(EqualFilter::from),
            exists_for_store_id: self.exists_for_store_id.map(EqualFilter::from),
        }
    }
}

#[derive(SimpleObject)]
pub struct MasterListConnector {
    total_count: u32,
    nodes: Vec<MasterListNode>,
}

impl MasterListConnector {
    pub fn from_domain(from: ListResult<MasterList>) -> MasterListConnector {
        MasterListConnector {
            total_count: from.count,
            nodes: from
                .rows
                .into_iter()
                .map(MasterListNode::from_domain)
                .collect(),
        }
    }
}

#[derive(Union)]
pub enum MasterListsResponse {
    Response(MasterListConnector),
}

pub fn master_lists(
    ctx: &Context<'_>,
    page: Option<PaginationInput>,
    filter: Option<MasterListFilterInput>,
    sort: Option<Vec<MasterListSortInput>>,
) -> Result<MasterListsResponse> {
    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;

    let master_lists = service_provider
        .master_list_service
        .get_master_lists(
            &service_context,
            page.map(PaginationOption::from),
            filter.map(|filter| filter.to_domain()),
            // Currently only one sort option is supported, use the first from the list.
            sort.map(|mut sort_list| sort_list.pop())
                .flatten()
                .map(|sort| sort.to_domain()),
        )
        .map_err(StandardGraphqlError::from_list_error)?;

    Ok(MasterListsResponse::Response(
        MasterListConnector::from_domain(master_lists),
    ))
}

#[cfg(test)]
mod test {
    use async_graphql::EmptyMutation;
    use graphql_core::{assert_graphql_query, test_helpers::setup_graphl_test};
    use repository::{
        mock::{mock_master_list_master_list_line_filter_test, MockDataInserts},
        MasterList, MasterListFilter, MasterListSort, StorageConnectionManager,
    };
    use repository::{EqualFilter, PaginationOption, SimpleStringFilter};
    use serde_json::{json, Value};

    use service::{
        master_list::MasterListServiceTrait,
        service_provider::{ServiceContext, ServiceProvider},
        ListError, ListResult,
    };

    use crate::GeneralQueries;

    type GetMasterLists = dyn Fn(
            Option<PaginationOption>,
            Option<MasterListFilter>,
            Option<MasterListSort>,
        ) -> Result<ListResult<MasterList>, ListError>
        + Sync
        + Send;

    pub struct TestService(pub Box<GetMasterLists>);

    impl MasterListServiceTrait for TestService {
        fn get_master_lists(
            &self,
            _: &ServiceContext,
            pagination: Option<PaginationOption>,
            filter: Option<MasterListFilter>,
            sort: Option<MasterListSort>,
        ) -> Result<ListResult<MasterList>, ListError> {
            (self.0)(pagination, filter, sort)
        }
    }

    pub fn service_provider(
        masterlist_service: TestService,
        connection_manager: &StorageConnectionManager,
    ) -> ServiceProvider {
        let mut service_provider = ServiceProvider::new(connection_manager.clone());
        service_provider.master_list_service = Box::new(masterlist_service);
        service_provider
    }

    #[actix_rt::test]
    async fn test_graphql_masterlists_success() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            GeneralQueries,
            EmptyMutation,
            "test_graphql_masterlists_success",
            MockDataInserts::all(),
        )
        .await;

        let query = r#"
        query {
            masterLists {
              ... on MasterListConnector {
                nodes {
                  id
                  name
                  code
                  description
                  lines {
                      nodes {
                          id
                          item {
                              id
                          }
                      }
                      totalCount
                  }
                }
                totalCount
              }
            }
        }
        "#;

        // Test single record
        let test_service = TestService(Box::new(|_, _, _| {
            Ok(ListResult {
                rows: vec![MasterList {
                    id: "master_list_master_list_line_filter_test".to_owned(),
                    name: "test_name".to_owned(),
                    code: "test_code".to_owned(),
                    description: "test_description".to_owned(),
                }],
                count: 1,
            })
        }));

        // TODO would prefer for loaders to be using service provider
        // in which case we would override both item and master list line service
        // and test it's mapping here, rather then from mock data
        let mock_data_lines = &mock_master_list_master_list_line_filter_test().lines;

        let lines: Vec<Value> = mock_data_lines
            .iter()
            .map(|line| {
                json!({
                    "id": line.id,
                    "item": {
                        "id": line.item_id
                    }
                })
            })
            .collect();

        let expected = json!({
              "masterLists": {
                  "nodes": [
                      {
                          "id": "master_list_master_list_line_filter_test",
                          "name": "test_name",
                          "code": "test_code",
                          "description": "test_description",
                          "lines": {
                              "nodes": lines,
                              "totalCount": lines.len()
                          }

                      },
                  ],
                  "totalCount": 1
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

        // Test no records

        let test_service = TestService(Box::new(|_, _, _| {
            Ok(ListResult {
                rows: Vec::new(),
                count: 0,
            })
        }));

        let expected = json!({
              "masterLists": {
                  "nodes": [

                  ],
                  "totalCount": 0
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

    #[actix_rt::test]
    async fn test_graphql_masterlists_filters() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            GeneralQueries,
            EmptyMutation,
            "test_graphql_masterlist_filters",
            MockDataInserts::all(),
        )
        .await;

        let query = r#"
        query(
            $filter: MasterListFilterInput
          ) {
            masterLists(filter: $filter) {
              __typename
            }
          }

        "#;

        let expected = json!({
              "masterLists": {
                  "__typename": "MasterListConnector"
              }
          }
        );

        // Test filter
        let test_service = TestService(Box::new(|_, filter, _| {
            assert_eq!(
                filter,
                Some(
                    MasterListFilter::new()
                        .id(EqualFilter::equal_to("test_id_filter"))
                        .name(SimpleStringFilter::equal_to("name_filter"))
                        .code(SimpleStringFilter::equal_to("code_filter"))
                        .description(SimpleStringFilter {
                            equal_to: Some("description_filter_1".to_owned()),
                            like: Some("description_filter_2".to_owned()),
                        })
                        .exists_for_name(SimpleStringFilter::like("exists_for_name_filter"))
                        .exists_for_name_id(EqualFilter::not_equal_to("test_name_id_filter"))
                )
            );
            Ok(ListResult::empty())
        }));

        let variables = json!({
          "filter": {
            "id": { "equalTo": "test_id_filter"},
            "name": {"equalTo": "name_filter" },
            "code": {"equalTo": "code_filter" },
            "description": {"equalTo": "description_filter_1", "like": "description_filter_2" },
            "existsForName": {"like": "exists_for_name_filter" },
            "existsForNameId": {"notEqualTo": "test_name_id_filter"}
          }
        });

        assert_graphql_query!(
            &settings,
            query,
            &Some(variables),
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );
    }
}
