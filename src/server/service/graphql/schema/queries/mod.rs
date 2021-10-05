pub mod pagination;

use crate::database::repository::{
    InvoiceLineRepository, InvoiceQueryRepository, RepositoryError, RequisitionRepository,
    StoreRepository,
};
use crate::database::schema::{InvoiceLineRow, RequisitionRow, StoreRow};
use crate::server::service::graphql::schema::types::{InvoiceLine, Requisition, Store};
use crate::server::service::graphql::ContextExt;

use super::types::{InvoiceList, InvoiceNode, ItemList, NameFilter, NameList, NameSortInput};
use async_graphql::*;
use pagination::Pagination;
pub struct Queries;

// RESPONSES
#[derive(Union)]
pub enum LoginStoreResponse {
    Error(ErrorWrapper<LoginStoreErrorInterface>),
    Response(StoreNode),
}

#[derive(Union)]
pub enum NamesResponse {
    Error(ErrorWrapper<ListErrorInterface>),
    Response(NameList),
}

#[derive(Union)]
pub enum LoginResponse {
    Error(ErrorWrapper<LoginErrorInterface>),
    UserInfo(UserInfo),
}

// ERROR WRAPPER
#[derive(SimpleObject)]
#[graphql(concrete(name = "LoginError", params(LoginErrorInterface)))]
#[graphql(concrete(name = "LoginStoreError", params(LoginStoreErrorInterface)))]
#[graphql(concrete(name = "ListError", params(ListErrorInterface)))]
pub struct ErrorWrapper<T: OutputType> {
    error: T,
}

// ERROR INTERFACES
#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum LoginErrorInterface {
    DBError(DBError),
    InvalidCredentials(InvalidCredentials),
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum LoginStoreErrorInterface {
    DBError(DBError),
    AuthenticationError(AuthenticationError),
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum ListErrorInterface {
    DBError(DBError),
    AuthenticationError(AuthenticationError),
    SortError(SortError),
    FilterError(FilterError),
    PaginationError(PaginationError),
}

// ERRORS
pub struct DBError(pub RepositoryError);

#[Object]
impl DBError {
    pub async fn description(&self) -> &'static str {
        "Dabase Error"
    }

    pub async fn full_error(&self) -> String {
        format!("{:#}", self.0)
    }
}

#[derive(SimpleObject)]
pub struct UserInfo {
    user: UserNode,
    token: String,
    logged_in_store: Option<StoreNode>,
    can_login_stores: Vec<StoreNode>,
}

#[derive(SimpleObject)]
pub struct UserNode {
    id: String,
    username: String,
    first_name: String,
    last_name: String,
    email: String,
}

#[derive(SimpleObject)]
pub struct StoreNode {
    id: String,
    name: String,
}

pub struct InvalidCredentials;

#[Object]
impl InvalidCredentials {
    pub async fn description(&self) -> &'static str {
        "Invalid Credentials"
    }
}

pub struct AuthenticationError;
#[Object]
impl AuthenticationError {
    pub async fn description(&self) -> &'static str {
        "Invalid token"
    }
}

pub struct SortError;
#[Object]
impl SortError {
    pub async fn description(&self) -> &'static str {
        "..."
    }
}

pub struct FilterError;
#[Object]
impl FilterError {
    pub async fn description(&self) -> &'static str {
        "..."
    }
}

pub struct PaginationError;
#[Object]
impl PaginationError {
    pub async fn description(&self) -> &'static str {
        "..."
    }
}

#[Object]
impl Queries {
    pub async fn login(
        &self,
        _ctx: &Context<'_>,
        username: String,
        password: String,
    ) -> LoginResponse {
        todo!()
    }

    pub async fn login_store(&self, _ctx: &Context<'_>, store_id: String) -> LoginStoreResponse {
        todo!()
    }

    #[allow(non_snake_case)]
    pub async fn apiVersion(&self) -> String {
        "1.0".to_string()
    }

    pub async fn names(
        &self,
        _ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<Pagination>,
        #[graphql(desc = "filters option")] filter: Option<NameFilter>,
        #[graphql(desc = "sort options")] sort: Option<Vec<NameSortInput>>,
    ) -> NamesResponse {
        todo!()
    }

    pub async fn items(
        &self,
        _ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<Pagination>,
    ) -> ItemList {
        ItemList { pagination: page }
    }

    // TODO return better error
    pub async fn invoice(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the invoice")] id: String,
    ) -> Result<InvoiceNode, RepositoryError> {
        let repository = ctx.get_repository::<InvoiceQueryRepository>();
        let invoice = repository.find_one_by_id(id.as_str()).await?;
        Ok(InvoiceNode::from(invoice))
    }

    pub async fn invoices(
        &self,
        _ctx: &Context<'_>,
        #[graphql(desc = "pagination (first and offset)")] page: Option<Pagination>,
    ) -> InvoiceList {
        InvoiceList { pagination: page }
    }

    pub async fn store(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the store")] id: String,
    ) -> Store {
        let store_repository = ctx.get_repository::<StoreRepository>();

        let store_row: StoreRow = store_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get store {}", id));

        Store { store_row }
    }

    pub async fn invoice_line(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the invoice line")] id: String,
    ) -> InvoiceLine {
        let invoice_line_repository = ctx.get_repository::<InvoiceLineRepository>();

        let invoice_line_row: InvoiceLineRow = invoice_line_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get invoice line {}", id));

        InvoiceLine { invoice_line_row }
    }

    pub async fn requisition(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the requisition")] id: String,
    ) -> Requisition {
        let requisition_repository = ctx.get_repository::<RequisitionRepository>();

        let requisition_row: RequisitionRow = requisition_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get requisition {}", id));

        Requisition { requisition_row }
    }
}
