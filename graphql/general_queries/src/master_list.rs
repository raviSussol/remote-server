use async_graphql::*;
use domain::{EqualFilter, PaginationOption, SimpleStringFilter};
use graphql_core::{
    generic_filters::{EqualFilterStringInput, SimpleStringFilterInput},
    pagination::PaginationInput,
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};
use graphql_types::types::MasterListNode;
use repository::{MasterList, MasterListSort, MasterListFilter};
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
