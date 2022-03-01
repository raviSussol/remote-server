use repository::{
    EqualFilter, RepositoryError, StocktakeLineFilter, StocktakeLineRepository,
    StocktakeRowRepository, StorageConnection, TransactionError,
};

use crate::{
    service_provider::ServiceContext,
    stocktake_line::delete::{delete_stocktake_line, DeleteStocktakeLineError},
    validate::check_store_id_matches,
};

use super::validate::{check_stocktake_exist, check_stocktake_not_finalised};

#[derive(Debug, PartialEq)]
pub enum DeleteStocktakeError {
    DatabaseError(RepositoryError),
    InvalidStore,
    StocktakeDoesNotExist,
    StocktakeLinesExist,
    CannotEditFinalised,
    LineDeleteError {
        line_id: String,
        error: DeleteStocktakeLineError,
    },
}

pub struct DeleteStocktakeInput {
    pub id: String,
}

fn validate(
    connection: &StorageConnection,
    store_id: &str,
    stocktake_id: &str,
) -> Result<(), DeleteStocktakeError> {
    let existing = match check_stocktake_exist(connection, stocktake_id)? {
        Some(existing) => existing,
        None => return Err(DeleteStocktakeError::StocktakeDoesNotExist),
    };
    if !check_store_id_matches(store_id, &existing.store_id) {
        return Err(DeleteStocktakeError::InvalidStore);
    }
    if !check_stocktake_not_finalised(&existing.status) {
        return Err(DeleteStocktakeError::CannotEditFinalised);
    }
    // TODO https://github.com/openmsupply/remote-server/issues/839
    // if !check_no_stocktake_lines_exist(connection, stocktake_id)? {
    //     return Err(DeleteStocktakeError::StocktakeLinesExist);
    // }
    Ok(())
}

/// Returns the id of the deleted stocktake
pub fn delete_stocktake(
    ctx: &ServiceContext,
    store_id: &str,
    stocktake_id: &str,
) -> Result<String, DeleteStocktakeError> {
    ctx.connection
        .transaction_sync(|connection| {
            validate(connection, store_id, stocktake_id)?;

            // TODO https://github.com/openmsupply/remote-server/issues/839
            let lines = StocktakeLineRepository::new(&connection).query_by_filter(
                StocktakeLineFilter::new().stocktake_id(EqualFilter::equal_to(&stocktake_id)),
            )?;
            for line in lines {
                delete_stocktake_line(ctx, store_id, &line.line.id).map_err(|error| {
                    DeleteStocktakeError::LineDeleteError {
                        line_id: line.line.id,
                        error,
                    }
                })?;
            }
            // End TODO

            StocktakeRowRepository::new(&connection).delete(stocktake_id)?;
            Ok(())
        })
        .map_err(|error: TransactionError<DeleteStocktakeError>| error.to_inner_error())?;
    Ok(stocktake_id.to_string())
}

impl From<RepositoryError> for DeleteStocktakeError {
    fn from(error: RepositoryError) -> Self {
        DeleteStocktakeError::DatabaseError(error)
    }
}

#[cfg(test)]
mod stocktake_test {

    use repository::{
        mock::{
            mock_stocktake_finalised_without_lines, mock_stocktake_without_lines, mock_store_a,
            MockDataInserts,
        },
        test_db::setup_all,
    };

    use crate::{service_provider::ServiceProvider, stocktake::delete::DeleteStocktakeError};

    #[actix_rt::test]
    async fn delete_stocktake() {
        let (_, _, connection_manager, _) =
            setup_all("delete_stocktake", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.stocktake_service;

        // error: stock does not exist
        let store_a = mock_stocktake_without_lines();
        let error = service
            .delete_stocktake(&context, &store_a.id, "invalid")
            .unwrap_err();
        assert_eq!(error, DeleteStocktakeError::StocktakeDoesNotExist);

        // error: invalid store
        let existing_stocktake = mock_stocktake_without_lines();
        let error = service
            .delete_stocktake(&context, "invalid", &existing_stocktake.id)
            .unwrap_err();
        assert_eq!(error, DeleteStocktakeError::InvalidStore);

        // TODO https://github.com/openmsupply/remote-server/issues/839
        // error: StocktakeLinesExist
        // let store_a = mock_store_a();
        // let stocktake_a = mock_stocktake_a();
        // let error = service
        //     .delete_stocktake(&context, &store_a.id, &stocktake_a.id)
        //     .unwrap_err();
        // assert_eq!(error, DeleteStocktakeError::StocktakeLinesExist);

        // error: CannotEditFinalised
        let store_a = mock_store_a();
        let stocktake = mock_stocktake_finalised_without_lines();
        let error = service
            .delete_stocktake(&context, &store_a.id, &stocktake.id)
            .unwrap_err();
        assert_eq!(error, DeleteStocktakeError::CannotEditFinalised);

        // success
        let store_a = mock_store_a();
        let existing_stocktake = mock_stocktake_without_lines();
        let deleted_stocktake_id = service
            .delete_stocktake(&context, &store_a.id, &existing_stocktake.id)
            .unwrap();
        assert_eq!(existing_stocktake.id, deleted_stocktake_id);
        assert_eq!(
            service
                .get_stocktake(&context, existing_stocktake.id)
                .unwrap(),
            None
        );
    }
}
