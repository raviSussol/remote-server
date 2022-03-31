use async_graphql::*;
use graphql_core::{
    simple_generic_errors::RecordNotFound, standard_graphql_error::StandardGraphqlError, ContextExt,
};
use graphql_types::types::ItemChartNode;
use service::requisition_line::chart::{
    ConsumptionHistoryOptions, RequisitionLineChartError, StockEvolutionOptions,
};

type ServiceError = RequisitionLineChartError;

#[derive(InputObject)]
pub struct ConsumptionOptionsInput {
    /// Defaults to 3 months
    amc_lookback_months: Option<u32>,
    /// Defaults to 12
    number_of_data_points: Option<u32>,
}

#[derive(InputObject)]
pub struct StockEvolutionOptionsInput {
    /// Defaults to 30, number of data points for historic stock on hand in stock evolution chart
    number_of_historic_data_points: Option<u32>,
    /// Defaults to 20, number of data points for projected stock on hand in stock evolution chart
    number_of_projected_data_points: Option<u32>,
}

#[derive(Interface)]
#[graphql(name = "RequisitionLineChartErrorInterface")]
#[graphql(field(name = "description", type = "String"))]
pub enum ChartErrorInterface {
    RecordNotFound(RecordNotFound),
}

#[derive(Union)]
#[graphql(name = "RequisitionLineChartResponse")]
pub enum ChartResponse {
    Response(ItemChartNode),
    Error(ChartError),
}

#[derive(SimpleObject)]
#[graphql(name = "RequisitionLineChartError")]
pub struct ChartError {
    pub error: ChartErrorInterface,
}

pub fn chart(
    ctx: &Context<'_>,
    store_id: &str,
    request_requisition_line_id: &str,
    consumption_options_input: Option<ConsumptionOptionsInput>,
    stock_evolution_options_input: Option<StockEvolutionOptionsInput>,
) -> Result<ChartResponse> {
    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;

    let result = match service_provider
        .requisition_line_service
        .get_requisition_line_chart(
            &service_context,
            store_id,
            request_requisition_line_id,
            consumption_options_input
                .map(|i| i.to_domain())
                .unwrap_or_default(),
            stock_evolution_options_input
                .map(|i| i.to_domain())
                .unwrap_or_default(),
        ) {
        Ok(requisition_line_chart) => {
            ChartResponse::Response(ItemChartNode::from_domain(requisition_line_chart))
        }
        Err(error) => ChartResponse::Error(ChartError {
            error: map_error(error)?,
        }),
    };

    Ok(result)
}

fn map_error(error: ServiceError) -> Result<ChartErrorInterface> {
    use StandardGraphqlError::*;
    let formatted_error = format!("{:#?}", error);

    let graphql_error = match error {
        // Structured Errors
        ServiceError::RequisitionLineDoesNotExist => {
            return Ok(ChartErrorInterface::RecordNotFound(RecordNotFound))
        }
        // Standard Graphql Errors
        ServiceError::RequisitionLineDoesNotBelongToCurrentStore => Forbidden(formatted_error),
        ServiceError::NotARequestRequisition => BadUserInput(formatted_error),
        ServiceError::DatabaseError(_) => InternalError(formatted_error),
    };

    Err(graphql_error.extend())
}

impl ConsumptionOptionsInput {
    fn to_domain(self) -> ConsumptionHistoryOptions {
        let ConsumptionOptionsInput {
            amc_lookback_months,
            number_of_data_points,
        } = self;
        let default = ConsumptionHistoryOptions::default();
        ConsumptionHistoryOptions {
            amc_lookback_months: amc_lookback_months.unwrap_or(default.amc_lookback_months),
            number_of_data_points: number_of_data_points.unwrap_or(default.number_of_data_points),
        }
    }
}

impl StockEvolutionOptionsInput {
    fn to_domain(self) -> StockEvolutionOptions {
        let StockEvolutionOptionsInput {
            number_of_historic_data_points,
            number_of_projected_data_points,
        } = self;
        let default = StockEvolutionOptions::default();
        StockEvolutionOptions {
            number_of_historic_data_points: number_of_historic_data_points
                .unwrap_or(default.number_of_historic_data_points),
            number_of_projected_data_points: number_of_projected_data_points
                .unwrap_or(default.number_of_projected_data_points),
        }
    }
}
#[cfg(test)]
mod graphql {
    use async_graphql::EmptyMutation;

    use chrono::NaiveDate;
    use graphql_core::assert_standard_graphql_error;
    use graphql_core::test_helpers::setup_graphl_test_with_data;
    use graphql_core::{assert_graphql_query, test_helpers::setup_graphl_test};
    use repository::mock::{mock_item_a, mock_name_a, MockData};
    use repository::schema::{
        InvoiceLineRow, InvoiceLineRowType, InvoiceRow, InvoiceRowType, NameRow,
        RequisitionLineRow, RequisitionRow, RequisitionRowType, StockLineRow, StoreRow,
    };
    use repository::{mock::MockDataInserts, StorageConnectionManager};
    use serde_json::json;

    use service::requisition_line::chart::{
        ConsumptionHistoryOptions, ItemChart, RequisitionLineChartError, StockEvolutionOptions,
    };
    use service::{
        requisition_line::RequisitionLineServiceTrait,
        service_provider::{ServiceContext, ServiceProvider},
    };
    use util::{inline_edit, inline_init, uuid};

    use crate::GeneralQueries;

    type ServiceError = RequisitionLineChartError;

    type GetRequisitionLineChart = dyn Fn(
            &str,
            &str,
            ConsumptionHistoryOptions,
            StockEvolutionOptions,
        ) -> Result<ItemChart, RequisitionLineChartError>
        + Sync
        + Send;

    pub struct TestService(pub Box<GetRequisitionLineChart>);

    impl RequisitionLineServiceTrait for TestService {
        fn get_requisition_line_chart(
            &self,
            _: &ServiceContext,
            store_id: &str,
            requisition_line_id: &str,
            consumption_history_options: ConsumptionHistoryOptions,
            stock_evolution_options: StockEvolutionOptions,
        ) -> Result<ItemChart, RequisitionLineChartError> {
            self.0(
                store_id,
                requisition_line_id,
                consumption_history_options,
                stock_evolution_options,
            )
        }
    }

    fn service_provider(
        test_service: TestService,
        connection_manager: &StorageConnectionManager,
    ) -> ServiceProvider {
        let mut service_provider = ServiceProvider::new(connection_manager.clone());
        service_provider.requisition_line_service = Box::new(test_service);
        service_provider
    }

    fn empty_variables() -> serde_json::Value {
        json!({
            "requestRequisitionLineId": "n/a",
            "storeId": "n/a"
        })
    }

    fn query() -> &'static str {
        r#"
        query MyQuery(
            $requestRequisitionLineId: String!
            $storeId: String!
            $consumptionOptionsInput: ConsumptionOptionsInput
            $stockEvolutionOptionsInput: StockEvolutionOptionsInput
          ) {
            requisitionLineChart(
              requestRequisitionLineId: $requestRequisitionLineId
              storeId: $storeId
              consumptionOptionsInput: $consumptionOptionsInput
              stockEvolutionOptionsInput: $stockEvolutionOptionsInput
            ) {
              __typename
              ... on RequisitionLineChartError {
                error {
                    __typename
                }
              }
              ... on ItemChartNode {
                consumptionHistory {
                    nodes {
                        consumption
                        averageMonthlyConsumption
                        date
                        isHistoric
                        isCurrent
                    }
                }
                stockEvolution {
                    nodes {
                        stockOnHand
                        date
                        minimumStockOnHand
                        maximumStockOnHand
                        isHistoric
                        isProjected
                    }
                }
                suggestedQuantityCalculation {
                    averageMonthlyConsumption
                    minimumStockOnHand
                    maximumStockOnHand
                    suggestedQuantity

                }
               calculationDate
              }
            }
          }
        "#
    }

    #[actix_rt::test]
    async fn test_graphql_get_requisition_line_chart_errors() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            GeneralQueries,
            EmptyMutation,
            "test_graphql_get_requisition_line_chart_errors",
            MockDataInserts::none(),
        )
        .await;

        // Test list error
        let test_service = TestService(Box::new(|_, _, _, _| {
            Err(ServiceError::RequisitionLineDoesNotExist)
        }));

        let expected = json!({
            "requisitionLineChart": {
                "error" : {
                    "__typename": "RecordNotFound"
                }
            }
        }
        );

        assert_graphql_query!(
            &settings,
            &query(),
            &Some(empty_variables()),
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        let test_service = TestService(Box::new(|_, _, _, _| {
            Err(ServiceError::RequisitionLineDoesNotBelongToCurrentStore)
        }));

        let expected_message = "Forbidden";
        assert_standard_graphql_error!(
            &settings,
            &query(),
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );

        let test_service = TestService(Box::new(|_, _, _, _| {
            Err(ServiceError::NotARequestRequisition)
        }));

        let expected_message = "Bad user input";
        assert_standard_graphql_error!(
            &settings,
            &query(),
            &Some(empty_variables()),
            &expected_message,
            None,
            Some(service_provider(test_service, &connection_manager))
        );
    }

    #[actix_rt::test]
    async fn test_graphql_get_requisition_line_chart_success() {
        let (_, _, connection_manager, settings) = setup_graphl_test(
            GeneralQueries,
            EmptyMutation,
            "test_graphql_get_requisition_line_chart_success",
            MockDataInserts::none(),
        )
        .await;

        // Test defaults
        let test_service = TestService(Box::new(|_, _, consumption_history, stock_evolution| {
            assert_eq!(stock_evolution, StockEvolutionOptions::default());
            assert_eq!(
                consumption_history,
                inline_init(|r: &mut ConsumptionHistoryOptions| {
                    r.amc_lookback_months = 20;
                })
            );
            Ok(ItemChart::default())
        }));

        let variables = json!({
            "requestRequisitionLineId": "n/a",
            "storeId": "n/a",
            "consumptionOptionsInput": {
                "amcLookbackMonths": 20
            }
        });

        let expected = json!({
            "requisitionLineChart": {
                "__typename": "ItemChartNode"
            }
        }
        );

        assert_graphql_query!(
            &settings,
            &query(),
            &Some(variables),
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );

        // Test all inputs
        let test_service = TestService(Box::new(
            |store_id, requisition_line_id, consumption_history, stock_evolution| {
                assert_eq!(
                    stock_evolution,
                    StockEvolutionOptions {
                        number_of_historic_data_points: 9,
                        number_of_projected_data_points: 10,
                    }
                );
                assert_eq!(
                    consumption_history,
                    ConsumptionHistoryOptions {
                        amc_lookback_months: 11,
                        number_of_data_points: 12
                    }
                );

                assert_eq!(store_id, "store_id");
                assert_eq!(requisition_line_id, "requisition_line_id");
                Ok(ItemChart::default())
            },
        ));

        let variables = json!({
            "requestRequisitionLineId": "requisition_line_id",
            "storeId": "store_id",
            "stockEvolutionOptionsInput": {
                "numberOfHistoricDataPoints": 9,
                "numberOfProjectedDataPoints": 10
            },
            "consumptionOptionsInput": {
                "amcLookbackMonths": 11,
                "numberOfDataPoints": 12
            }
        });

        let expected = json!({
            "requisitionLineChart": {
                "__typename": "ItemChartNode"
            }
        }
        );

        assert_graphql_query!(
            &settings,
            &query(),
            &Some(variables),
            &expected,
            Some(service_provider(test_service, &connection_manager))
        );
    }

    #[actix_rt::test]
    async fn chart_data() {
        fn name() -> NameRow {
            inline_init(|r: &mut NameRow| {
                r.id = "store".to_string();
            })
        }

        fn store() -> StoreRow {
            StoreRow {
                id: "store".to_string(),
                name_id: name().id,
                code: "store".to_string(),
            }
        }

        fn stock_line() -> StockLineRow {
            inline_init(|r: &mut StockLineRow| {
                r.id = "stcok_line".to_string();
                r.item_id = mock_item_a().id;
                r.store_id = store().id;
                r.pack_size = 1;
                r.available_number_of_packs = 20;
                r.total_number_of_packs = 20;
            })
        }

        fn requisition() -> RequisitionRow {
            inline_init(|r: &mut RequisitionRow| {
                r.id = "requisition".to_string();
                r.store_id = store().id;
                r.requisition_number = 333;
                r.min_months_of_stock = 30.0 / 100.0;
                r.max_months_of_stock = 200.0 / 100.0;
                r.name_id = mock_name_a().id;
                r.expected_delivery_date = Some(NaiveDate::from_ymd(2021, 01, 12));
                r.r#type = RequisitionRowType::Request;
            })
        }

        fn requisition_line() -> RequisitionLineRow {
            inline_init(|r: &mut RequisitionLineRow| {
                r.id = "requisition_line".to_string();
                r.requisition_id = requisition().id;
                r.item_id = mock_item_a().id;
                r.snapshot_datetime = Some(NaiveDate::from_ymd(2021, 01, 02).and_hms(0, 0, 0));
                r.average_monthly_consumption = 100;
                r.available_stock_on_hand = 20;
                r.suggested_quantity = 180;
            })
        }

        fn consumption_point() -> MockData {
            let invoice_id = uuid::uuid();
            inline_init(|r: &mut MockData| {
                r.invoices = vec![inline_init(|r: &mut InvoiceRow| {
                    r.id = invoice_id.clone();
                    r.store_id = store().id;
                    r.name_id = mock_name_a().id;
                    r.r#type = InvoiceRowType::OutboundShipment;
                })];
                r.invoice_lines = vec![inline_init(|r: &mut InvoiceLineRow| {
                    r.id = format!("{}line", invoice_id);
                    r.invoice_id = invoice_id.clone();
                    r.item_id = mock_item_a().id;
                    r.r#type = InvoiceLineRowType::StockOut;
                    r.stock_line_id = None;
                    r.pack_size = 1;
                })];
            })
        }

        let (_, _, connection_manager, settings) = setup_graphl_test_with_data(
            GeneralQueries,
            EmptyMutation,
            "chart_data",
            MockDataInserts::all(),
            inline_init(|r: &mut MockData| {
                r.names = vec![name()];
                r.stores = vec![store()];
                r.requisitions = vec![requisition()];
                r.requisition_lines = vec![requisition_line()];
                r.stock_lines = vec![stock_line()];
            })
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 90;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 11, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 85;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 10, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 110;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 09, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 130;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 08, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 70;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 07, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 80;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 06, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 85;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 05, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 100;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 04, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 75;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 03, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 60;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 02, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 80;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 01, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 80;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2019, 12, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 80;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2019, 11, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 80;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2019, 11, 01).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 80;
                u
            }))
            // stock history
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2021, 01, 02).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 2;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 30).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 5;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 29).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 5;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 25).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 2;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 23).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 1;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 24).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 5;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 19).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 4;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 18).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 3;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 15).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 5;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 12).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 3;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 10).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 2;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 9).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 8;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 5).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 5;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 4).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 6;
                u
            }))
            .join(inline_edit(&consumption_point(), |mut u| {
                u.invoices[0].picked_datetime =
                    Some(NaiveDate::from_ymd(2020, 12, 3).and_hms(0, 0, 0));
                u.invoice_lines[0].number_of_packs = 3;
                u
            })),
        )
        .await;

        let expected = json!({
           "requisitionLineChart":{
              "__typename":"ItemChartNode",
              "calculationDate":"2021-01-02",
              "consumptionHistory":{
                 "nodes":[
                    {
                       "averageMonthlyConsumption":80,
                       "consumption":80,
                       "date":"2020-02-29",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":73,
                       "consumption":60,
                       "date":"2020-03-31",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":72,
                       "consumption":75,
                       "date":"2020-04-30",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":77,
                       "consumption":100,
                       "date":"2020-05-31",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":86,
                       "consumption":85,
                       "date":"2020-06-30",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":87,
                       "consumption":80,
                       "date":"2020-07-31",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":77,
                       "consumption":70,
                       "date":"2020-08-31",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":92,
                       "consumption":130,
                       "date":"2020-09-30",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":102,
                       "consumption":110,
                       "date":"2020-10-31",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":108,
                       "consumption":85,
                       "date":"2020-11-30",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":112,
                       "consumption":147,
                       "date":"2020-12-31",
                       "isCurrent":false,
                       "isHistoric":true
                    },
                    {
                       "averageMonthlyConsumption":100,
                       "consumption":100,
                       "date":"2021-01-31",
                       "isCurrent":false,
                       "isHistoric":false
                    }
                 ]
              },
              "suggestedQuantityCalculation":{
                 "averageMonthlyConsumption":100,
                 "maximumStockOnHand":200,
                 "minimumStockOnHand":30,
                 "suggestedQuantity":180
              }
           }
        });

        let variables = json!({
            "requestRequisitionLineId": "requisition_line",
            "storeId": "store",
        });

        assert_graphql_query!(&settings, &query(), &Some(variables), &expected, None);
    }
}
