use chrono::{NaiveDate, Utc};
use repository::schema::StocktakeStatus;
use repository::EqualFilter;
use repository::{
    schema::{NumberRowType, StocktakeRow},
    RepositoryError, Stocktake, StocktakeFilter, StocktakeRepository, StocktakeRowRepository,
    StorageConnection,
};

use crate::user_account::get_default_user_id;
use crate::{number::next_number, service_provider::ServiceContext, validate::check_store_exists};

use super::query::get_stocktake;

#[derive(Default, Debug, PartialEq)]
pub struct InsertStocktakeInput {
    pub id: String,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub stocktake_date: Option<NaiveDate>,
    pub is_locked: Option<bool>,
}

#[derive(Debug, PartialEq)]
pub enum InsertStocktakeError {
    DatabaseError(RepositoryError),
    InternalError(String),
    StocktakeAlreadyExists,
    InvalidStore,
}

fn check_stocktake_does_not_exist(
    connection: &StorageConnection,
    id: &str,
) -> Result<bool, RepositoryError> {
    let count = StocktakeRepository::new(connection)
        .count(Some(StocktakeFilter::new().id(EqualFilter::equal_to(id))))?;
    Ok(count == 0)
}

fn validate(
    connection: &StorageConnection,
    store_id: &str,
    stocktake: &InsertStocktakeInput,
) -> Result<(), InsertStocktakeError> {
    if !check_stocktake_does_not_exist(connection, &stocktake.id)? {
        return Err(InsertStocktakeError::StocktakeAlreadyExists);
    }
    if !check_store_exists(connection, store_id)? {
        return Err(InsertStocktakeError::InvalidStore);
    }
    Ok(())
}

fn generate(
    connection: &StorageConnection,
    store_id: &str,
    InsertStocktakeInput {
        id,
        comment,
        description,
        stocktake_date,
        is_locked,
    }: InsertStocktakeInput,
) -> Result<StocktakeRow, RepositoryError> {
    let stocktake_number = next_number(connection, &NumberRowType::Stocktake, store_id)?;

    Ok(StocktakeRow {
        id,
        stocktake_number,
        comment,
        description,
        stocktake_date,
        status: StocktakeStatus::New,
        created_datetime: Utc::now().naive_utc(),
        user_id: get_default_user_id(),
        store_id: store_id.to_string(),
        is_locked: is_locked.unwrap_or(false),
        // Default
        finalised_datetime: None,
        inventory_adjustment_id: None,
    })
}

pub fn insert_stocktake(
    ctx: &ServiceContext,
    store_id: &str,
    input: InsertStocktakeInput,
) -> Result<Stocktake, InsertStocktakeError> {
    let result = ctx
        .connection
        .transaction_sync(|connection| {
            validate(connection, store_id, &input)?;
            let new_stocktake = generate(connection, store_id, input)?;
            StocktakeRowRepository::new(&connection).upsert_one(&new_stocktake)?;

            let stocktake = get_stocktake(ctx, new_stocktake.id)?;
            stocktake.ok_or(InsertStocktakeError::InternalError(
                "Failed to read the just inserted stocktake!".to_string(),
            ))
        })
        .map_err(|error| error.to_inner_error())?;
    Ok(result)
}

impl From<RepositoryError> for InsertStocktakeError {
    fn from(error: RepositoryError) -> Self {
        InsertStocktakeError::DatabaseError(error)
    }
}

#[cfg(test)]
mod test {
    use chrono::{NaiveDate, Utc};
    use repository::{
        mock::{mock_stocktake_a, mock_store_a, MockDataInserts},
        schema::{StocktakeRow, StocktakeStatus},
        test_db::setup_all,
        StocktakeRowRepository,
    };
    use util::{inline_edit, inline_init};

    use crate::{
        service_provider::ServiceProvider,
        stocktake::insert::{InsertStocktakeError, InsertStocktakeInput},
        user_account::get_default_user_id,
    };

    #[actix_rt::test]
    async fn insert_stocktake() {
        let (_, connection, connection_manager, _) =
            setup_all("insert_stocktake", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.stocktake_service;

        // error: stocktake already exists
        let store_a = mock_store_a();
        let existing_stocktake = mock_stocktake_a();
        let error = service
            .insert_stocktake(
                &context,
                &store_a.id,
                inline_init(|i: &mut InsertStocktakeInput| {
                    i.id = existing_stocktake.id;
                }),
            )
            .unwrap_err();
        assert_eq!(error, InsertStocktakeError::StocktakeAlreadyExists);

        // error: store does not exist
        let error = service
            .insert_stocktake(
                &context,
                "invalid",
                inline_init(|i: &mut InsertStocktakeInput| i.id = "new_stocktake".to_string()),
            )
            .unwrap_err();
        assert_eq!(error, InsertStocktakeError::InvalidStore);

        // success
        let before_insert = Utc::now().naive_utc();

        let store_a = mock_store_a();
        service
            .insert_stocktake(
                &context,
                &store_a.id,
                InsertStocktakeInput {
                    id: "new_stocktake".to_string(),
                    comment: Some("comment".to_string()),
                    description: Some("description".to_string()),
                    stocktake_date: Some(NaiveDate::from_ymd(2020, 01, 02)),
                    is_locked: Some(true),
                },
            )
            .unwrap();

        let after_insert = Utc::now().naive_utc();

        let new_row = StocktakeRowRepository::new(&connection)
            .find_one_by_id("new_stocktake")
            .unwrap()
            .unwrap();

        assert_eq!(
            new_row,
            inline_edit(&new_row, |mut i: StocktakeRow| {
                i.user_id = get_default_user_id();
                i.id = "new_stocktake".to_string();
                i.comment = Some("comment".to_string());
                i.description = Some("description".to_string());
                i.stocktake_date = Some(NaiveDate::from_ymd(2020, 01, 02));
                i.is_locked = true;
                i.status = StocktakeStatus::New;
                i.store_id = store_a.id;
                i
            }),
        );

        assert!(
            new_row.created_datetime > before_insert && new_row.created_datetime < after_insert
        );
    }
}
