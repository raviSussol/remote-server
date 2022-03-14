use async_graphql::{Context, Enum, InputObject, Result, SimpleObject, Union};
use graphql_core::{
    generic_filters::{EqualFilterStringInput, SimpleStringFilterInput},
    pagination::PaginationInput,
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};
use graphql_types::types::NameNode;
use repository::{EqualFilter, PaginationOption, SimpleStringFilter};
use repository::{Name, NameFilter, NameSort, NameSortField};
use service::ListResult;

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
#[graphql(rename_items = "camelCase")]
pub enum NameSortFieldInput {
    Name,
    Code,
}

#[derive(InputObject)]
pub struct NameSortInput {
    /// Sort query result by `key`
    key: NameSortFieldInput,
    /// Sort query result is sorted descending or ascending (if not provided the default is
    /// ascending)
    desc: Option<bool>,
}

#[derive(InputObject, Clone)]
pub struct NameFilterInput {
    pub id: Option<EqualFilterStringInput>,
    /// Filter by name
    pub name: Option<SimpleStringFilterInput>,
    /// Filter by code
    pub code: Option<SimpleStringFilterInput>,
    /// Filter by customer property
    pub is_customer: Option<bool>,
    /// Filter by supplier property
    pub is_supplier: Option<bool>,
    /// Is this name a store
    pub is_store: Option<bool>,
    /// Code of the store if store is linked to name
    pub store_code: Option<SimpleStringFilterInput>,
    // Visibility in current store (based on store_id parameter and existance of name_store_join record)
    pub is_visible: Option<bool>,
    // Show system names (defaults to false)
    // System names don't have name_store_join thus if queried with true filter, is_visible filter should also be true or null
    pub is_system_name: Option<bool>,
}

#[derive(SimpleObject)]
pub struct NameConnector {
    total_count: u32,
    nodes: Vec<NameNode>,
}

#[derive(Union)]
pub enum NamesResponse {
    Response(NameConnector),
}

pub fn get_names(
    ctx: &Context<'_>,
    store_id: &str,
    page: Option<PaginationInput>,
    filter: Option<NameFilterInput>,
    sort: Option<Vec<NameSortInput>>,
) -> Result<NamesResponse> {
    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;

    let names = service_provider
        .general_service
        .get_names(
            &service_context,
            &store_id,
            page.map(PaginationOption::from),
            filter.map(|filter| filter.to_domain()),
            // Currently only one sort option is supported, use the first from the list.
            sort.map(|mut sort_list| sort_list.pop())
                .flatten()
                .map(|sort| sort.to_domain()),
        )
        .map_err(StandardGraphqlError::from_list_error)?;

    Ok(NamesResponse::Response(NameConnector::from_domain(names)))
}

impl NameFilterInput {
    pub fn to_domain(self) -> NameFilter {
        let NameFilterInput {
            id,
            name,
            code,
            is_customer,
            is_supplier,
            is_store,
            store_code,
            is_visible,
            is_system_name,
        } = self;

        NameFilter {
            id: id.map(EqualFilter::from),
            name: name.map(SimpleStringFilter::from),
            code: code.map(SimpleStringFilter::from),
            store_code: store_code.map(SimpleStringFilter::from),
            is_customer,
            is_supplier,
            is_store,
            is_visible,
            is_system_name: is_system_name.or(Some(false)),
        }
    }
}

impl NameConnector {
    pub fn from_domain(names: ListResult<Name>) -> NameConnector {
        NameConnector {
            total_count: names.count,
            nodes: names.rows.into_iter().map(NameNode::from_domain).collect(),
        }
    }
}

impl NameSortInput {
    pub fn to_domain(self) -> NameSort {
        use NameSortField as to;
        use NameSortFieldInput as from;
        let key = match self.key {
            from::Name => to::Name,
            from::Code => to::Code,
        };

        NameSort {
            key,
            desc: self.desc,
        }
    }
}
