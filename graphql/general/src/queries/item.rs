use async_graphql::*;
use graphql_core::{
    generic_filters::{EqualFilterStringInput, SimpleStringFilterInput},
    map_filter,
    pagination::PaginationInput,
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};
use graphql_types::types::{ItemConnector, ItemNodeType};
use repository::{EqualFilter, PaginationOption, SimpleStringFilter};
use repository::{ItemFilter, ItemSort, ItemSortField};
use service::item::get_items;

#[derive(Enum, Copy, Clone, PartialEq, Eq)]
#[graphql(remote = "repository::ItemSortField")]
#[graphql(rename_items = "camelCase")]
pub enum ItemSortFieldInput {
    Name,
    Code,
}

#[derive(InputObject)]
pub struct ItemSortInput {
    /// Sort query result by `key`
    key: ItemSortFieldInput,
    /// Sort query result is sorted descending or ascending (if not provided the default is
    /// ascending)
    desc: Option<bool>,
}

#[derive(InputObject, Clone)]
pub struct EqualFilterItemTypeInput {
    pub equal_to: Option<ItemNodeType>,
    pub equal_any: Option<Vec<ItemNodeType>>,
    pub not_equal_to: Option<ItemNodeType>,
}

#[derive(InputObject, Clone)]
pub struct ItemFilterInput {
    pub id: Option<EqualFilterStringInput>,
    pub name: Option<SimpleStringFilterInput>,
    pub r#type: Option<EqualFilterItemTypeInput>,
    pub code: Option<SimpleStringFilterInput>,
    pub is_visible: Option<bool>,
}

#[derive(Union)]
pub enum ItemsResponse {
    Response(ItemConnector),
}

pub fn items(
    ctx: &Context<'_>,
    page: Option<PaginationInput>,
    filter: Option<ItemFilterInput>,
    sort: Option<Vec<ItemSortInput>>,
) -> Result<ItemsResponse> {
    let connection_manager = ctx.get_connection_manager();
    let items = get_items(
        connection_manager,
        page.map(PaginationOption::from),
        filter.map(|filter| filter.to_domain()),
        // Currently only one sort option is supported, use the first from the list.
        sort.and_then(|mut sort_list| sort_list.pop())
            .map(|sort| sort.to_domain()),
    )
    .map_err(StandardGraphqlError::from_list_error)?;

    Ok(ItemsResponse::Response(ItemConnector::from_domain(items)))
}

impl ItemFilterInput {
    pub fn to_domain(self) -> ItemFilter {
        let ItemFilterInput {
            id,
            name,
            r#type,
            code,
            is_visible,
        } = self;

        ItemFilter {
            id: id.map(EqualFilter::from),
            name: name.map(SimpleStringFilter::from),
            code: code.map(SimpleStringFilter::from),
            r#type: r#type.map(|t| map_filter!(t, ItemNodeType::to_domain)),
            is_visible,
        }
    }
}

impl ItemSortInput {
    pub fn to_domain(self) -> ItemSort {
        use ItemSortField as to;
        use ItemSortFieldInput as from;
        let key = match self.key {
            from::Name => to::Name,
            from::Code => to::Code,
        };

        ItemSort {
            key,
            desc: self.desc,
        }
    }
}
