use crate::domain::item::ItemFilter;
use crate::domain::name::NameFilter;
use crate::domain::PaginationOption;
use crate::server::service::graphql::ContextExt;
use crate::service::invoice::get_invoices;
use crate::service::item::get_items;
use crate::service::name::get_names;
use crate::{domain::invoice::InvoiceFilter, service::invoice::get_invoice};

use super::types::{
    convert_sort, InvoiceFilterInput, InvoiceResponse, InvoiceSortInput, InvoicesResponse,
    ItemFilterInput, ItemSortInput, ItemsResponse, NameFilterInput, NameSortInput, NamesResponse,
    PaginationInput,
};
use async_graphql::*;

pub struct Queries;

#[Object]
impl Queries {
    #[allow(non_snake_case)]
    pub async fn apiVersion(&self) -> String {
        "1.0".to_string()
    }

    pub async fn names(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<PaginationInput>,
        #[graphql(desc = "filters option")] filter: Option<NameFilterInput>,
        #[graphql(desc = "sort options (only first sort input is evaluated for this endpoint)")]
        sort: Option<Vec<NameSortInput>>,
    ) -> NamesResponse {
        let connection_pool = ctx.get_connection_pool();
        get_names(
            connection_pool,
            page.map(PaginationOption::from),
            filter.map(NameFilter::from),
            convert_sort(sort),
        )
        .into()
    }

    pub async fn items(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<PaginationInput>,
        #[graphql(desc = "filters option")] filter: Option<ItemFilterInput>,
        #[graphql(desc = "sort options (only first sort input is evaluated for this endpoint)")]
        sort: Option<Vec<ItemSortInput>>,
    ) -> ItemsResponse {
        let connection_pool = ctx.get_connection_pool();
        get_items(
            connection_pool,
            page.map(PaginationOption::from),
            filter.map(ItemFilter::from),
            convert_sort(sort),
        )
        .into()
    }

    pub async fn invoice(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the invoice")] id: String,
    ) -> InvoiceResponse {
        let connection_pool = ctx.get_connection_pool();
        get_invoice(connection_pool, id).into()
    }

    pub async fn invoices(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<PaginationInput>,
        #[graphql(desc = "filters option")] filter: Option<InvoiceFilterInput>,
        #[graphql(desc = "sort options (only first sort input is evaluated for this endpoint)")]
        sort: Option<Vec<InvoiceSortInput>>,
    ) -> InvoicesResponse {
        let connection_pool = ctx.get_connection_pool();
        get_invoices(
            connection_pool,
            page.map(PaginationOption::from),
            filter.map(InvoiceFilter::from),
            convert_sort(sort),
        )
        .into()
    }
}
