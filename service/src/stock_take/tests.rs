#[cfg(test)]
mod stock_take_test {
    use chrono::Utc;
    use repository::{
        mock::{
            mock_stock_take_a, mock_stock_take_finalized_without_lines,
            mock_stock_take_without_lines, mock_store_a, MockDataInserts,
        },
        test_db::setup_all,
    };

    use crate::{
        service_provider::ServiceProvider,
        stock_take::{
            delete::DeleteStockTakeError,
            insert::{InsertStockTakeError, InsertStockTakeInput},
        },
    };

    #[actix_rt::test]
    async fn insert_stock_take() {
        let (_, _, connection_manager, _) =
            setup_all("insert_stock_take", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.stock_take_service;

        // error: stock take already exists
        let store_a = mock_store_a();
        let existing_stock_take = mock_stock_take_a();
        let error = service
            .insert_stock_take(
                &context,
                &store_a.id,
                InsertStockTakeInput {
                    id: existing_stock_take.id,
                    comment: None,
                    description: None,
                    created_datetime: Utc::now().naive_utc(),
                },
            )
            .unwrap_err();
        assert_eq!(error, InsertStockTakeError::StockTakeAlreadyExists);

        // error: store does not exist
        let error = service
            .insert_stock_take(
                &context,
                "invalid",
                InsertStockTakeInput {
                    id: "new_stock_take".to_string(),
                    comment: None,
                    description: None,
                    created_datetime: Utc::now().naive_utc(),
                },
            )
            .unwrap_err();
        assert_eq!(error, InsertStockTakeError::InvalidStore);

        // success
        let store_a = mock_store_a();
        service
            .insert_stock_take(
                &context,
                &store_a.id,
                InsertStockTakeInput {
                    id: "new_stock_take".to_string(),
                    comment: None,
                    description: None,
                    created_datetime: Utc::now().naive_utc(),
                },
            )
            .unwrap();
    }

    #[actix_rt::test]
    async fn delete_stock_take() {
        let (_, _, connection_manager, _) =
            setup_all("delete_stock_take", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.stock_take_service;

        // error: stock does not exist
        let store_a = mock_stock_take_without_lines();
        let error = service
            .delete_stock_take(&context, &store_a.id, "invalid")
            .unwrap_err();
        assert_eq!(error, DeleteStockTakeError::StockTakeDoesNotExist);

        // error: invalid store
        let existing_stock_take = mock_stock_take_without_lines();
        let error = service
            .delete_stock_take(&context, "invalid", &existing_stock_take.id)
            .unwrap_err();
        assert_eq!(error, DeleteStockTakeError::InvalidStore);

        // error: StockTakeLinesExist
        let store_a = mock_store_a();
        let stock_take_a = mock_stock_take_a();
        let error = service
            .delete_stock_take(&context, &store_a.id, &stock_take_a.id)
            .unwrap_err();
        assert_eq!(error, DeleteStockTakeError::StockTakeLinesExist);

        // error: CannotEditFinalised
        let store_a = mock_store_a();
        let stock_take = mock_stock_take_finalized_without_lines();
        let error = service
            .delete_stock_take(&context, &store_a.id, &stock_take.id)
            .unwrap_err();
        assert_eq!(error, DeleteStockTakeError::CannotEditFinalised);

        // success
        let store_a = mock_store_a();
        let existing_stock_take = mock_stock_take_without_lines();
        let deleted_stock_take_id = service
            .delete_stock_take(&context, &store_a.id, &existing_stock_take.id)
            .unwrap();
        assert_eq!(existing_stock_take.id, deleted_stock_take_id);
        assert_eq!(
            service
                .get_stock_take(&context, existing_stock_take.id)
                .unwrap(),
            None
        );
    }
}