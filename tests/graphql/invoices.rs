#![allow(where_clauses_object_safety)]

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
            service::graphql::config as graphql_config,
        },
        util::test_db,
    };

    use cynic::{
        selection_set::{field, float, map1, map2, string, vec},
        Operation, QueryRoot, SelectionSet,
    };

    use serde::Serialize;

    #[allow(dead_code)]
    #[derive(Serialize)]
    struct InvoicesResult {
        data: InvoicesRoot,
    }

    #[derive(Serialize)]
    struct InvoicesRoot {
        invoices: InvoicesConnection,
    }

    impl QueryRoot for InvoicesRoot {}

    #[allow(dead_code)]
    #[derive(Serialize)]
    struct InvoicesConnection {
        nodes: Vec<Invoice>,
    }

    #[allow(dead_code)]
    #[derive(Serialize)]
    struct Invoice {
        id: String,
        pricing: InvoicePricing,
    }

    #[allow(dead_code)]
    #[allow(non_snake_case)]
    #[derive(Serialize)]
    struct InvoicePricing {
        totalAfterTax: f64,
    }

    fn build_invoices_query() -> Operation<'static, InvoicesRoot> {
        let select_invoice_pricing: SelectionSet<'_, InvoicePricing, InvoicePricing> = map1(
            #[allow(non_snake_case)]
            |totalAfterTax| InvoicePricing { totalAfterTax },
            field("totalAfterTax", vec![], float()),
        );

        let select_invoice: SelectionSet<'_, Invoice, Invoice> = map2(
            |id, pricing| Invoice { id, pricing },
            field("id", vec![], string()),
            field("pricing", vec![], select_invoice_pricing),
        );

        let select_invoices_connection: SelectionSet<'_, InvoicesConnection, InvoicesConnection> =
            map1(
                |nodes| InvoicesConnection { nodes },
                field("nodes", vec![], vec(select_invoice)),
            );

        let select_invoices_root: SelectionSet<'_, InvoicesRoot, InvoicesRoot> = map1(
            |invoices| InvoicesRoot { invoices },
            field("invoices", vec![], select_invoices_connection),
        );

        Operation::query(select_invoices_root)
    }

    #[actix_rt::test]
    async fn test_graphql_invoices_query() {
        let settings = test_db::get_test_settings("omsupply-database-gql-invoices-query");
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

        let invoices_query = build_invoices_query();

        let request = actix_web::test::TestRequest::post()
            .header("content-type", "application/json")
            .set_json(&invoices_query)
            .uri("/graphql")
            .to_request();

        let response = actix_web::test::read_response(&mut app, request).await;
        let result = String::from_utf8(response.to_vec()).expect("Failed to parse result");

        let expected = serde_json::to_string(&InvoicesResult {
            data: InvoicesRoot {
                invoices: InvoicesConnection {
                    nodes: (&mock_invoices)
                        .into_iter()
                        .map(|invoice| Invoice {
                            id: invoice.id.to_owned(),
                            pricing: InvoicePricing {
                                totalAfterTax: (&mock_invoice_lines).into_iter().fold(
                                    0.0,
                                    |acc, invoice_line| {
                                        if invoice_line.invoice_id == invoice.id {
                                            acc + invoice_line.total_after_tax
                                        } else {
                                            acc
                                        }
                                    },
                                ),
                            },
                        })
                        .collect(),
                },
            },
        })
        .unwrap();

        assert_eq!(result, expected);
    }
}
