use crate::{
    number::next_number,
    requisition::{common::check_requisition_exists, query::get_requisition},
    service_provider::ServiceContext,
};
use chrono::Utc;
use domain::{
    name::{Name, NameFilter},
    EqualFilter,
};
use repository::{
    schema::{NumberRowType, RequisitionRow, RequisitionRowStatus, RequisitionRowType},
    NameQueryRepository, RepositoryError, Requisition, RequisitionRowRepository, StorageConnection,
};

#[derive(Debug, PartialEq)]
pub struct InsertRequestRequisition {
    pub id: String,
    pub other_party_id: String,
    pub colour: Option<String>,
    pub their_reference: Option<String>,
    pub comment: Option<String>,
    pub max_months_of_stock: f64,
    pub threshold_months_of_stock: f64,
}

#[derive(Debug, PartialEq)]

pub enum InsertRequestRequisitionError {
    RequisitionAlreadyExists,
    OtherPartyNotASupplier,
    OtherPartyDoesNotExist,
    OtherPartyIsThisStore,
    OtherPartyIsNotAStore,
    NewlyCreatedRequisitionDoesNotExist,
    DatabaseError(RepositoryError),
}

type OutError = InsertRequestRequisitionError;

pub fn insert_request_requisition(
    ctx: &ServiceContext,
    store_id: &str,
    input: InsertRequestRequisition,
) -> Result<Requisition, OutError> {
    let requisition = ctx
        .connection
        .transaction_sync(|connection| {
            validate(connection, store_id, &input)?;
            let new_requisition = generate(connection, store_id, input)?;
            RequisitionRowRepository::new(&connection).upsert_one(&new_requisition)?;

            get_requisition(ctx, None, &new_requisition.id)
                .map_err(|error| OutError::DatabaseError(error))?
                .ok_or(OutError::NewlyCreatedRequisitionDoesNotExist)
        })
        .map_err(|error| error.to_inner_error())?;
    Ok(requisition)
}

fn validate(
    connection: &StorageConnection,
    store_id: &str,
    input: &InsertRequestRequisition,
) -> Result<(), OutError> {
    if let Some(_) = check_requisition_exists(connection, &input.id)? {
        return Err(OutError::RequisitionAlreadyExists);
    }

    let other_party = check_other_party_exists(connection, &input.other_party_id)?
        .ok_or(OutError::OtherPartyDoesNotExist)?;

    if !other_party.is_supplier {
        return Err(OutError::OtherPartyNotASupplier);
    }

    let other_party_store_id = other_party
        .store_id
        .ok_or(OutError::OtherPartyIsNotAStore)?;

    if store_id == other_party_store_id {
        return Err(OutError::OtherPartyIsThisStore);
    }

    Ok(())
}

fn generate(
    connection: &StorageConnection,
    store_id: &str,
    InsertRequestRequisition {
        id,
        other_party_id,
        colour,
        comment,
        their_reference,
        max_months_of_stock,
        threshold_months_of_stock,
    }: InsertRequestRequisition,
) -> Result<RequisitionRow, RepositoryError> {
    let result = RequisitionRow {
        id,
        requisition_number: next_number(connection, &NumberRowType::RequestRequisition, &store_id)?,
        name_id: other_party_id,
        store_id: store_id.to_owned(),
        r#type: RequisitionRowType::Request,
        status: RequisitionRowStatus::Draft,
        created_datetime: Utc::now().naive_utc(),
        sent_datetime: None,
        finalised_datetime: None,
        colour,
        comment,
        their_reference,
        max_months_of_stock,
        threshold_months_of_stock,
        linked_requisition_id: None,
    };

    Ok(result)
}

impl From<RepositoryError> for InsertRequestRequisitionError {
    fn from(error: RepositoryError) -> Self {
        InsertRequestRequisitionError::DatabaseError(error)
    }
}

pub fn check_other_party_exists(
    connection: &StorageConnection,
    other_party_id: &str,
) -> Result<Option<Name>, RepositoryError> {
    // TODO store_id check
    let result = NameQueryRepository::new(connection)
        .query_by_filter(NameFilter::new().id(EqualFilter::equal_to(other_party_id)))?
        .pop();

    Ok(result)
}

#[cfg(test)]
mod test_insert {
    use chrono::Utc;
    use repository::{
        mock::{
            mock_name_a, mock_name_store_b, mock_name_store_c, mock_request_draft_requisition,
            MockDataInserts,
        },
        schema::{RequisitionRow, RequisitionRowStatus, RequisitionRowType},
        test_db::setup_all,
        RequisitionRowRepository,
    };

    use crate::{
        requisition::request_requisition::{
            InsertRequestRequisition, InsertRequestRequisitionError as ServiceError,
        },
        service_provider::ServiceProvider,
    };

    #[actix_rt::test]
    async fn insert_request_requisition_errors() {
        let (_, _, connection_manager, _) =
            setup_all("insert_request_requisition_errors", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.requisition_service;

        // RequisitionAlreadyExists
        assert_eq!(
            service.insert_request_requisition(
                &context,
                "store_a",
                InsertRequestRequisition {
                    id: mock_request_draft_requisition().id,
                    other_party_id: "n/a".to_owned(),
                    colour: None,
                    their_reference: None,
                    comment: None,
                    max_months_of_stock: 1.0,
                    threshold_months_of_stock: 0.5,
                },
            ),
            Err(ServiceError::RequisitionAlreadyExists)
        );

        // OtherPartyNotASupplier
        assert_eq!(
            service.insert_request_requisition(
                &context,
                "store_a",
                InsertRequestRequisition {
                    id: "new_request_requisition".to_owned(),
                    other_party_id: mock_name_store_b().id,
                    colour: None,
                    their_reference: None,
                    comment: None,
                    max_months_of_stock: 1.0,
                    threshold_months_of_stock: 0.5,
                },
            ),
            Err(ServiceError::OtherPartyNotASupplier)
        );

        // OtherPartyDoesNotExist
        assert_eq!(
            service.insert_request_requisition(
                &context,
                "store_a",
                InsertRequestRequisition {
                    id: "new_request_requisition".to_owned(),
                    other_party_id: "invalid".to_owned(),
                    colour: None,
                    their_reference: None,
                    comment: None,
                    max_months_of_stock: 1.0,
                    threshold_months_of_stock: 0.5,
                },
            ),
            Err(ServiceError::OtherPartyDoesNotExist)
        );

        // OtherPartyNotASupplier
        assert_eq!(
            service.insert_request_requisition(
                &context,
                "store_a",
                InsertRequestRequisition {
                    id: "new_request_requisition".to_owned(),
                    other_party_id: mock_name_store_b().id,
                    colour: None,
                    their_reference: None,
                    comment: None,
                    max_months_of_stock: 1.0,
                    threshold_months_of_stock: 0.5,
                },
            ),
            Err(ServiceError::OtherPartyNotASupplier)
        );

        // OtherPartyIsNotAStore
        assert_eq!(
            service.insert_request_requisition(
                &context,
                "store_c",
                InsertRequestRequisition {
                    id: "new_request_requisition".to_owned(),
                    other_party_id: mock_name_a().id,
                    colour: None,
                    their_reference: None,
                    comment: None,
                    max_months_of_stock: 1.0,
                    threshold_months_of_stock: 0.5,
                },
            ),
            Err(ServiceError::OtherPartyIsNotAStore)
        );

        // OtherPartyIsThisStore
        assert_eq!(
            service.insert_request_requisition(
                &context,
                "store_c",
                InsertRequestRequisition {
                    id: "new_request_requisition".to_owned(),
                    other_party_id: mock_name_store_c().id,
                    colour: None,
                    their_reference: None,
                    comment: None,
                    max_months_of_stock: 1.0,
                    threshold_months_of_stock: 0.5,
                },
            ),
            Err(ServiceError::OtherPartyIsThisStore)
        );
    }

    #[actix_rt::test]
    async fn insert_request_requisition_success() {
        let (_, connection, connection_manager, _) =
            setup_all("insert_request_requisition_success", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.requisition_service;

        let before_insert = Utc::now().naive_utc();

        let result = service
            .insert_request_requisition(
                &context,
                "store_a",
                InsertRequestRequisition {
                    id: "new_request_requisition".to_owned(),
                    other_party_id: mock_name_store_c().id,
                    colour: Some("new colour".to_owned()),
                    their_reference: Some("new their_reference".to_owned()),
                    comment: Some("new comment".to_owned()),
                    max_months_of_stock: 1.0,
                    threshold_months_of_stock: 0.5,
                },
            )
            .unwrap();

        let after_insert = Utc::now().naive_utc();

        let RequisitionRow {
            id,
            requisition_number: _,
            name_id,
            store_id,
            r#type,
            status,
            created_datetime,
            sent_datetime,
            finalised_datetime,
            colour,
            comment,
            their_reference,
            max_months_of_stock,
            threshold_months_of_stock,
            linked_requisition_id,
        } = RequisitionRowRepository::new(&connection)
            .find_one_by_id(&result.requisition_row.id)
            .unwrap()
            .unwrap();

        assert_eq!(id, "new_request_requisition".to_owned());

        assert_eq!(name_id, mock_name_store_c().id);
        assert_eq!(colour, Some("new colour".to_owned()));
        assert_eq!(their_reference, Some("new their_reference".to_owned()));
        assert_eq!(comment, Some("new comment".to_owned()));
        assert_eq!(max_months_of_stock, 1.0);
        assert_eq!(threshold_months_of_stock, 0.5);
        assert_eq!(store_id, "store_a".to_owned());
        assert_eq!(r#type, RequisitionRowType::Request);
        assert_eq!(status, RequisitionRowStatus::Draft);
        assert!(created_datetime > before_insert && created_datetime < after_insert);
        assert_eq!(sent_datetime, None);
        assert_eq!(finalised_datetime, None);
        assert_eq!(linked_requisition_id, None);
    }
}