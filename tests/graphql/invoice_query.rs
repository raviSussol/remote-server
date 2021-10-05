mod graphql {
    use remote_server::{
        database::{
            loader::get_loaders,
            mock::{
                mock_invoice_lines, mock_invoices, mock_items, mock_names, mock_stock_lines,
                mock_stores,
            },
            repository::{
                get_repositories, InvoiceLineRepository, InvoiceRepository, ItemRepository,
                NameRepository, StockLineRepository, StoreRepository,
            },
            schema::{InvoiceLineRow, InvoiceRow, ItemRow, NameRow, StockLineRow, StoreRow},
        },
        server::{
            data::{LoaderRegistry, RepositoryRegistry},
            service::graphql::{config as graphql_config, schema::types::InvoiceStatus},
        },
        util::test_db,
    };

    use cynic::{
        selection_set::{field, integer, map1, map2, map3, string, vec},
        Argument, Operation, QueryRoot, SelectionSet,
    };

    use serde::Serialize;

    #[derive(Serialize)]
    struct InvoiceQueryResult {
        data: InvoiceQueryRoot,
    }

    #[derive(Serialize)]
    struct InvoiceQueryRoot {
        invoice: InvoiceQuery,
    }

    impl QueryRoot for InvoiceQueryRoot {}

    #[derive(Serialize)]
    struct InvoiceQuery {
        id: String,
        lines: InvoiceQueryLinesConnection,
        status: InvoiceStatus,
    }

    #[derive(Serialize)]
    struct InvoiceQueryLinesConnection {
        nodes: Vec<InvoiceQueryLine>,
    }

    #[derive(Serialize)]
    #[allow(non_snake_case)]
    struct InvoiceQueryLine {
        id: String,
        stockLine: InvoiceQueryLineStockLine,
    }

    #[derive(Serialize)]
    #[allow(non_snake_case)]
    struct InvoiceQueryLineStockLine {
        availableNumberOfPacks: i32,
    }

    fn build_invoice_query(id: &str) -> Operation<'static, InvoiceQueryRoot> {
        let select_invoice_query_line_stock_line: SelectionSet<
            '_,
            InvoiceQueryLineStockLine,
            InvoiceQueryLineStockLine,
        > = map1(
            #[allow(non_snake_case)]
            |availableNumberOfPacks| InvoiceQueryLineStockLine {
                availableNumberOfPacks,
            },
            field("availableNumberOfPacks", vec![], integer()),
        );

        let select_invoice_query_line: SelectionSet<'_, InvoiceQueryLine, InvoiceQueryLine> = map2(
            #[allow(non_snake_case)]
            |id, stockLine| InvoiceQueryLine { id, stockLine },
            field("id", vec![], string()),
            field("stockLine", vec![], select_invoice_query_line_stock_line),
        );

        let select_invoice_query_lines: SelectionSet<
            '_,
            InvoiceQueryLinesConnection,
            InvoiceQueryLinesConnection,
        > = map1(
            |nodes| InvoiceQueryLinesConnection { nodes },
            field("nodes", vec![], vec(select_invoice_query_line)),
        );

        let select_invoice_query: SelectionSet<'_, InvoiceQuery, InvoiceQuery> = map3(
            |id, status, lines| InvoiceQuery {
                id,
                status: match &status[..] {
                    "Confirmed" => InvoiceStatus::Confirmed,
                    "Finalised" => InvoiceStatus::Finalised,
                    _ => InvoiceStatus::Draft,
                },
                lines,
            },
            field("id", vec![], string()),
            field("status", vec![], string()),
            field("lines", vec![], select_invoice_query_lines),
        );

        let select_invoice_query_root: SelectionSet<'_, InvoiceQueryRoot, InvoiceQueryRoot> = map1(
            |invoice| InvoiceQueryRoot { invoice },
            field(
                "invoice",
                vec![Argument::new("id", "String", id)],
                select_invoice_query,
            ),
        );

        Operation::query(select_invoice_query_root)
    }

    #[actix_rt::test]
    async fn test_graphql_invoice_query() {
        let settings = test_db::get_test_settings("omsupply-database-gql-invoice-query");
        test_db::setup(&settings.database).await;

        let repositories = get_repositories(&settings).await;
        let loaders = get_loaders(&settings).await;

        let name_repository = repositories.get::<NameRepository>().unwrap();
        let store_repository = repositories.get::<StoreRepository>().unwrap();
        let item_repository = repositories.get::<ItemRepository>().unwrap();
        let stock_repository = repositories.get::<StockLineRepository>().unwrap();
        let invoice_repository = repositories.get::<InvoiceRepository>().unwrap();
        let invoice_line_repository = repositories.get::<InvoiceLineRepository>().unwrap();

        let mock_names: Vec<NameRow> = mock_names();
        let mock_stores: Vec<StoreRow> = mock_stores();
        let mock_items: Vec<ItemRow> = mock_items();
        let mock_stocks: Vec<StockLineRow> = mock_stock_lines();
        let mock_invoices: Vec<InvoiceRow> = mock_invoices();
        let mock_invoice_lines: Vec<InvoiceLineRow> = mock_invoice_lines();

        for name in &mock_names {
            name_repository.insert_one(&name).await.unwrap();
        }

        for store in &mock_stores {
            store_repository.insert_one(&store).await.unwrap();
        }

        for item in &mock_items {
            item_repository.insert_one(&item).await.unwrap();
        }

        for stock_line in &mock_stocks {
            stock_repository.insert_one(&stock_line).await.unwrap();
        }

        for invoice in &mock_invoices {
            invoice_repository.insert_one(&invoice).await.unwrap();
        }

        for invoice_line in &mock_invoice_lines {
            invoice_line_repository
                .insert_one(&invoice_line)
                .await
                .unwrap();
        }

        let repository_registry = RepositoryRegistry { repositories };
        let loader_registry = LoaderRegistry { loaders };

        let repository_registry = actix_web::web::Data::new(repository_registry);
        let loader_registry = actix_web::web::Data::new(loader_registry);

        let mut app = actix_web::test::init_service(
            actix_web::App::new()
                .data(repository_registry.clone())
                .data(loader_registry.clone())
                .configure(graphql_config(repository_registry, loader_registry)),
        )
        .await;

        let invoice = &mock_invoices.first().expect("Failed to find mock invoice");

        let invoice_query = build_invoice_query(&invoice.id);

        let request = actix_web::test::TestRequest::post()
            .header("content-type", "application/json")
            .set_json(&invoice_query)
            .uri("/graphql")
            .to_request();

        let response = actix_web::test::read_response(&mut app, request).await;
        let result = String::from_utf8(response.to_vec()).expect("Failed to parse response");

        let expected = serde_json::to_string(&InvoiceQueryResult {
            data: InvoiceQueryRoot {
                invoice: InvoiceQuery {
                    id: invoice.id.clone(),
                    lines: InvoiceQueryLinesConnection {
                        nodes: (&mock_invoice_lines)
                            .into_iter()
                            .filter(|&invoice_line| invoice_line.invoice_id == invoice.id)
                            .map(|invoice_line| InvoiceQueryLine {
                                id: invoice_line.id.clone(),
                                stockLine: InvoiceQueryLineStockLine {
                                    availableNumberOfPacks: (&mock_stocks)
                                        .into_iter()
                                        .find(|&stock_line| {
                                            if let Some(stock_line_id) = &invoice_line.stock_line_id
                                            {
                                                &stock_line.id == stock_line_id
                                            } else {
                                                false
                                            }
                                        })
                                        .unwrap()
                                        .available_number_of_packs
                                        + invoice_line.available_number_of_packs,
                                },
                            })
                            .collect(),
                    },
                    status: invoice.status.clone().into(),
                },
            },
        })
        .unwrap();

        assert_eq!(result, expected);

        let invoice_query = build_invoice_query("invalid");

        let request = actix_web::test::TestRequest::post()
            .header("content-type", "application/json")
            .set_json(&invoice_query)
            .uri("/graphql")
            .to_request();

        let response = actix_web::test::read_response(&mut app, request).await;
        let body = String::from_utf8(response.to_vec()).expect("Failed to parse response");

        assert!(body.contains("row not found"));
    }
}
