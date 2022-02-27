use graphql_core::loader::{get_loaders, LoaderRegistry};
use server::settings::Settings;

use graphql::config as graphql_config;
use repository::get_storage_connection_manager;
use service::{auth_data::AuthData, service_provider::ServiceProvider, token_bucket::TokenBucket};

use std::sync::RwLock;

use actix_web::web::Data;
use serde_json::Value;

mod common;
// mod inbound_shipment_delete;
// mod inbound_shipment_insert;
// mod inbound_shipment_line_delete;
// mod inbound_shipment_line_insert;
// mod inbound_shipment_line_update;
// mod inbound_shipment_update;
mod invoice_by_number;
mod invoice_loaders;
mod invoices;
mod item_stats;
mod items;
mod location_delete;
mod location_insert;
mod location_update;
mod locations;
mod master_lists;
mod names;
// mod outbound_shipment_delete;
// mod outbound_shipment_insert;
// mod outbound_shipment_line_delete;
// mod outbound_shipment_line_insert;
// mod outbound_shipment_line_update;
// mod outbound_shipment_update;
mod pagination;
mod requisition;
mod stocktake_batch;
mod stocktake_delete;
mod stocktake_insert;
mod stocktake_line_delete;
mod stocktake_line_insert;
mod stocktake_line_update;
mod stocktake_query;
mod stocktake_update;
mod unallocated_line;

async fn run_gql_query(
    settings: &Settings,
    query: &str,
    variables: &Option<serde_json::Value>,
    service_provider_override: Option<ServiceProvider>,
) -> serde_json::Value {
    let connection_manager = get_storage_connection_manager(&settings.database);
    let connection_manager_data = actix_web::web::Data::new(connection_manager.clone());

    let service_provider_data = actix_web::web::Data::new(match service_provider_override {
        Some(service_provider) => service_provider,
        None => ServiceProvider::new(connection_manager.clone()),
    });

    let loaders = get_loaders(&connection_manager, service_provider_data.clone()).await;
    let loader_registry = actix_web::web::Data::new(LoaderRegistry { loaders });

    let auth_data = Data::new(AuthData {
        auth_token_secret: settings.auth.token_secret.to_owned(),
        token_bucket: RwLock::new(TokenBucket::new()),
        // TODO: configure ssl
        debug_no_ssl: true,
        debug_no_access_control: true,
    });

    let mut app = actix_web::test::init_service(
        actix_web::App::new()
            .data(connection_manager_data.clone())
            .data(loader_registry.clone())
            .configure(graphql_config(
                connection_manager_data,
                loader_registry,
                service_provider_data,
                auth_data,
            )),
    )
    .await;

    let mut payload: String;
    if let Some(variables) = variables {
        payload = format!(
            "{{\"query\":\"{}\",\"variables\":{}}}",
            query,
            variables.to_string()
        );
    } else {
        payload = format!("{{\"query\":\"{}\"}}", query);
    }
    payload = payload.replace("\n", "");

    let req = actix_web::test::TestRequest::post()
        .header("content-type", "application/json")
        .set_payload(payload)
        .uri("/graphql")
        .to_request();

    let res = actix_web::test::read_response(&mut app, req).await;
    let body = String::from_utf8(res.to_vec()).expect("Failed to parse response");
    serde_json::from_str::<Value>(&body).expect(body.as_str())
}

async fn assert_gql_not_found(
    settings: &Settings,
    query: &str,
    variables: &Option<serde_json::Value>,
    service_provider_override: Option<ServiceProvider>,
) -> serde_json::Value {
    let actual = run_gql_query(settings, query, variables, service_provider_override).await;
    let error_message = actual["data"].to_string();
    assert!(error_message.contains("RecordNotFound"));
    actual
}

macro_rules! assert_graphql_query {
    ($settings:expr, $query:expr, $variables:expr, $expected_inner:expr, $service_provider_override:expr) => {{
        let actual = crate::graphql::run_gql_query(
            $settings,
            $query,
            $variables,
            $service_provider_override,
        )
        .await;

        match actual.get("errors").and_then(serde_json::Value::as_array) {
            Some(errors) => {
                if !errors.is_empty() {
                    panic!("Request failed with standard error(s): {}",
                        serde_json::to_string_pretty(errors).unwrap());
                }
            },
            None => {}
        }
        let expected = serde_json::json!(
            {
                "data": $expected_inner,
            }
        );

        // Inclusive means only match fields in rhs against lhs (lhs can have more fields)
        let config = assert_json_diff::Config::new(assert_json_diff::CompareMode::Inclusive);

        match assert_json_diff::assert_json_matches_no_panic(&actual, &expected, config) {
            Ok(_) => assert!(true),
            Err(error) => {
                panic!(
                    "\n{}\n**actual**\n{}\n**expected**\n{}\n**query**\n{}",
                    error,
                    serde_json::to_string_pretty(&actual).unwrap(),
                    serde_json::to_string_pretty(&expected).unwrap(),
                    $query
                );
            }
        }
    }};
}

macro_rules! assert_standard_graphql_error {
    // expected_etensions should be an Option<serde_json::json>>
    ($settings:expr, $query:expr, $variables:expr, $expected_message:expr, $expected_extensions:expr, $service_provider_override:expr) => {{
        let actual = crate::graphql::run_gql_query(
            $settings,
            $query,
            $variables,
            $service_provider_override
        )
        .await;

        let expected_with_message = serde_json::json!(
            {
                "errors": [{
                    "message": $expected_message,
                    // Need to check that extensions are indeed present,
                    // and if expected_extensions is not, None check content of extensions
                    "extensions": $expected_extensions.unwrap_or(serde_json::json!({}))
                }]
            }
        );
        // Inclusive means only match fields in rhs against lhs (lhs can have more fields)
        let config = assert_json_diff::Config::new(assert_json_diff::CompareMode::Inclusive);

        match assert_json_diff::assert_json_matches_no_panic(
            &actual,
            &expected_with_message,
            config,
        ) {
            Ok(_) => assert!(true),
            Err(error) => {
                panic!(
                    "\n{}\n**actual**\n{}\n**expected**\n{}\n**query**\n{}",
                    error,
                    serde_json::to_string_pretty(&actual).unwrap(),
                    serde_json::to_string_pretty(&expected_with_message).unwrap(),
                    $query
                );
            }
        }
    }};
}

pub(crate) use assert_graphql_query;
pub(crate) use assert_standard_graphql_error;
