#![allow(where_clauses_object_safety)]

mod graphql {
    use remote_server::{
        database::{
            loader::get_loaders,
            mock::{mock_names, mock_requisitions, mock_stores},
            repository::{
                get_repositories, NameRepository, RequisitionRepository, StoreRepository,
            },
            schema::{NameRow, RequisitionRow, StoreRow},
        },
        server::{
            data::{LoaderRegistry, RepositoryRegistry},
            service::graphql::config as graphql_config,
        },
        util::test_db,
    };

    use cynic::{
        selection_set::{field, map1, string},
        Argument, Operation, QueryRoot, SelectionSet,
    };

    use serde::Serialize;

    #[derive(Serialize)]
    struct RequisitionResult {
        data: RequisitionRoot,
    }

    #[derive(Serialize)]
    struct RequisitionRoot {
        requisition: Requisition,
    }

    impl QueryRoot for RequisitionRoot {}

    #[derive(Serialize)]
    struct Requisition {
        id: String,
    }

    fn build_requisition_query(id: &str) -> Operation<'static, RequisitionRoot> {
        let select_requisition: SelectionSet<'_, Requisition, Requisition> =
            map1(|id| Requisition { id }, field("id", vec![], string()));

        let select_requisition_root: SelectionSet<'_, RequisitionRoot, RequisitionRoot> = map1(
            |requisition| RequisitionRoot { requisition },
            field(
                "requisition",
                vec![Argument::new("id", "String", id)],
                select_requisition,
            ),
        );

        Operation::query(select_requisition_root)
    }

    #[actix_rt::test]
    async fn get_requisition_by_id_is_success() {
        let settings = test_db::get_test_settings("omsupply-database-simple-repository-test");
        test_db::setup(&settings.database).await;

        let repositories = get_repositories(&settings).await;
        let loaders = get_loaders(&settings).await;

        let mock_names: Vec<NameRow> = mock_names();
        let mock_stores: Vec<StoreRow> = mock_stores();
        let mock_requisitions: Vec<RequisitionRow> = mock_requisitions();

        let name_repository = repositories.get::<NameRepository>().unwrap();
        let store_repository = repositories.get::<StoreRepository>().unwrap();
        let requisition_repository = repositories.get::<RequisitionRepository>().unwrap();

        for name in &mock_names {
            name_repository.insert_one(&name).await.unwrap();
        }

        for store in &mock_stores {
            store_repository.insert_one(&store).await.unwrap();
        }

        for requisition in &mock_requisitions {
            requisition_repository
                .insert_one(&requisition)
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

        let requisition_id = mock_requisitions
            .first()
            .expect("Failed to find mock requisition")
            .id
            .to_owned();

        let requisition_query = build_requisition_query(&requisition_id);

        let request = actix_web::test::TestRequest::post()
            .header("content-type", "application/json")
            .set_json(&requisition_query)
            .uri("/graphql")
            .to_request();

        let response = actix_web::test::read_response(&mut app, request).await;
        let result = String::from_utf8(response.to_vec()).expect("Failed to parse response");

        let expected = serde_json::to_string(&RequisitionResult {
            data: RequisitionRoot {
                requisition: Requisition { id: requisition_id },
            },
        })
        .unwrap();

        assert_eq!(result, expected);
    }
}
