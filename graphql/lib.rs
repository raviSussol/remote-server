#[cfg(test)]
mod tests;

use actix_web::web::{self, Data};
use actix_web::HttpResponse;
use actix_web::{guard, HttpRequest};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::MergedObject;
use async_graphql::{EmptySubscription, SchemaBuilder};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use graphql_batch_mutations::BatchMutations;
use graphql_core::auth_data_from_request;
use graphql_core::loader::LoaderRegistry;
use graphql_general::{GeneralMutations, GeneralQueries};
use graphql_invoice::{InvoiceMutations, InvoiceQueries};
use graphql_invoice_line::InvoiceLineMutations;
use graphql_location::{LocationMutations, LocationQueries};
use graphql_reports::ReportQueries;
use graphql_requisition::{RequisitionMutations, RequisitionQueries};
use graphql_requisition_line::RequisitionLineMutations;
use graphql_stocktake::{StocktakeMutations, StocktakeQueries};
use graphql_stocktake_line::StocktakeLineMutations;

use repository::StorageConnectionManager;
use service::auth_data::AuthData;
use service::service_provider::ServiceProvider;

#[derive(MergedObject, Default, Clone)]
pub struct FullQuery(
    pub InvoiceQueries,
    pub LocationQueries,
    pub StocktakeQueries,
    pub GeneralQueries,
    pub RequisitionQueries,
    pub ReportQueries,
);

#[derive(MergedObject, Default, Clone)]
pub struct FullMutation(
    pub InvoiceMutations,
    pub InvoiceLineMutations,
    pub LocationMutations,
    pub StocktakeMutations,
    pub StocktakeLineMutations,
    pub BatchMutations,
    pub GeneralMutations,
    pub RequisitionMutations,
    pub RequisitionLineMutations,
);

pub type Schema = async_graphql::Schema<FullQuery, FullMutation, async_graphql::EmptySubscription>;
type Builder = SchemaBuilder<FullQuery, FullMutation, EmptySubscription>;

pub fn full_query() -> FullQuery {
    FullQuery(
        InvoiceQueries,
        LocationQueries,
        StocktakeQueries,
        GeneralQueries,
        RequisitionQueries,
        ReportQueries,
    )
}

pub fn full_mutation() -> FullMutation {
    FullMutation(
        InvoiceMutations,
        InvoiceLineMutations,
        LocationMutations,
        StocktakeMutations,
        StocktakeLineMutations,
        BatchMutations,
        GeneralMutations,
        RequisitionMutations,
        RequisitionLineMutations,
    )
}

pub fn build_schema() -> Builder {
    Schema::build(full_query(), full_mutation(), EmptySubscription)
}

pub fn config(
    connection_manager: Data<StorageConnectionManager>,
    loader_registry: Data<LoaderRegistry>,
    service_provider: Data<ServiceProvider>,
    auth_data: Data<AuthData>,
) -> impl FnOnce(&mut actix_web::web::ServiceConfig) {
    |cfg| {
        let schema = build_schema()
            .data(connection_manager)
            .data(loader_registry)
            .data(service_provider)
            .data(auth_data)
            .finish();
        cfg.app_data(Data::new(schema))
            .service(web::resource("/graphql").guard(guard::Post()).to(
                |schema: Data<Schema>, http_req, req: GraphQLRequest| {
                    graphql(schema, http_req, req)
                },
            ))
            .service(
                web::resource("/playground")
                    .guard(guard::Get())
                    .to(playground),
            );
    }
}

async fn playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/")))
}

async fn graphql(
    schema: Data<Schema>,
    http_req: HttpRequest,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let user_data = auth_data_from_request(&http_req);
    let query = req.into_inner().data(user_data);
    schema.execute(query).await.into()
}

#[cfg(test)]
mod test {
    use graphql_core::{assert_graphql_query, test_helpers::setup_graphl_test};
    use repository::mock::MockDataInserts;
    use serde_json::json;

    use crate::{full_mutation, full_query};

    #[actix_rt::test]
    async fn test_graphql_version() {
        // This test should also checks that there are no duplicate types (which will be a panic when schema is built)
        let (_, _, _, settings) = setup_graphl_test(
            full_query(),
            full_mutation(),
            "graphql_requisition_user_loader",
            MockDataInserts::none(),
        )
        .await;
        let expected = json!({
            "apiVersion": "1.0"
        });

        let query = r#"
        query {
            apiVersion
        }
        "#;

        assert_graphql_query!(&settings, &query, &None, expected, None);
    }
}
