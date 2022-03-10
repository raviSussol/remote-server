use crate::{
    requisition::common::{check_requisition_exists, generate_requisition_user_id_update},
    requisition_line::{common::check_requisition_line_exists, query::get_requisition_line},
    service_provider::ServiceContext,
};

use repository::{
    schema::{RequisitionLineRow, RequisitionRow, RequisitionRowStatus, RequisitionRowType},
    RepositoryError, RequisitionLine, RequisitionLineRowRepository, RequisitionRowRepository,
    StorageConnection,
};
use util::inline_edit;

#[derive(Debug, PartialEq, Default)]
pub struct UpdateResponseRequisitionLine {
    pub id: String,
    pub supply_quantity: Option<u32>,
    pub comment: Option<String>,
}

#[derive(Debug, PartialEq)]

pub enum UpdateResponseRequisitionLineError {
    RequisitionLineDoesNotExist,
    NotThisStoreRequisition,
    CannotEditRequisition,
    NotAResponseRequisition,
    UpdatedRequisitionLineDoesNotExist,
    RequisitionDoesNotExist,
    DatabaseError(RepositoryError),
}

type OutError = UpdateResponseRequisitionLineError;

pub fn update_response_requisition_line(
    ctx: &ServiceContext,
    store_id: &str,
    user_id: &str,
    input: UpdateResponseRequisitionLine,
) -> Result<RequisitionLine, OutError> {
    let requisition_line = ctx
        .connection
        .transaction_sync(|connection| {
            let (requisition_row, requisition_line_row) = validate(connection, store_id, &input)?;
            let (requisition_row_option, updated_requisition_line_row) =
                generate(user_id, requisition_row, requisition_line_row, input);

            RequisitionLineRowRepository::new(&connection)
                .upsert_one(&updated_requisition_line_row)?;

            if let Some(requisition_row) = requisition_row_option {
                RequisitionRowRepository::new(&connection).upsert_one(&requisition_row)?;
            }

            get_requisition_line(ctx, &updated_requisition_line_row.id)
                .map_err(|error| OutError::DatabaseError(error))?
                .ok_or(OutError::UpdatedRequisitionLineDoesNotExist)
        })
        .map_err(|error| error.to_inner_error())?;
    Ok(requisition_line)
}

fn validate(
    connection: &StorageConnection,
    store_id: &str,
    input: &UpdateResponseRequisitionLine,
) -> Result<(RequisitionRow, RequisitionLineRow), OutError> {
    let requisition_line_row = check_requisition_line_exists(connection, &input.id)?
        .ok_or(OutError::RequisitionLineDoesNotExist)?;

    let requisition_row =
        check_requisition_exists(connection, &requisition_line_row.requisition_id)?
            .ok_or(OutError::RequisitionDoesNotExist)?;

    if requisition_row.store_id != store_id {
        return Err(OutError::NotThisStoreRequisition);
    }

    if requisition_row.r#type != RequisitionRowType::Response {
        return Err(OutError::NotAResponseRequisition);
    }

    if requisition_row.status != RequisitionRowStatus::New {
        return Err(OutError::CannotEditRequisition);
    }

    Ok((requisition_row, requisition_line_row))
}

fn generate(
    user_id: &str,
    existing_requisition_row: RequisitionRow,
    existing: RequisitionLineRow,
    UpdateResponseRequisitionLine {
        id: _,
        supply_quantity: updated_supply_quantity,
        comment: updated_comment,
    }: UpdateResponseRequisitionLine,
) -> (Option<RequisitionRow>, RequisitionLineRow) {
    let requisition_line_row = inline_edit(&existing, |mut u| {
        u.supply_quantity = updated_supply_quantity.unwrap_or(u.supply_quantity as u32) as i32;
        u.comment = updated_comment.or(u.comment);
        u
    });

    (
        generate_requisition_user_id_update(user_id, existing_requisition_row),
        requisition_line_row,
    )
}

impl From<RepositoryError> for UpdateResponseRequisitionLineError {
    fn from(error: RepositoryError) -> Self {
        UpdateResponseRequisitionLineError::DatabaseError(error)
    }
}

#[cfg(test)]
mod test {
    use repository::{
        mock::{
            mock_finalised_request_requisition_line, mock_new_response_requisition_test,
            mock_sent_request_requisition_line, mock_user_account_b, MockDataInserts,
        },
        test_db::setup_all,
        RequisitionLineRowRepository, RequisitionRowRepository,
    };
    use util::{inline_edit, inline_init};

    use crate::{
        requisition_line::response_requisition_line::{
            UpdateResponseRequisitionLine, UpdateResponseRequisitionLineError as ServiceError,
        },
        service_provider::ServiceProvider,
    };

    #[actix_rt::test]
    async fn update_response_requisition_line_errors() {
        let (_, _, connection_manager, _) = setup_all(
            "update_response_requisition_line_errors",
            MockDataInserts::all(),
        )
        .await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.requisition_line_service;

        // RequisitionLineDoesNotExist
        assert_eq!(
            service.update_response_requisition_line(
                &context,
                "store_a",
                "n/a",
                inline_init(|r: &mut UpdateResponseRequisitionLine| {
                    r.id = "invalid".to_owned();
                }),
            ),
            Err(ServiceError::RequisitionLineDoesNotExist)
        );

        // NotThisStoreRequisition
        assert_eq!(
            service.update_response_requisition_line(
                &context,
                "store_b",
                "n/a",
                inline_init(|r: &mut UpdateResponseRequisitionLine| {
                    r.id = mock_new_response_requisition_test().lines[0].id.clone();
                }),
            ),
            Err(ServiceError::NotThisStoreRequisition)
        );

        // CannotEditRequisition
        assert_eq!(
            service.update_response_requisition_line(
                &context,
                "store_a",
                "n/a",
                inline_init(|r: &mut UpdateResponseRequisitionLine| {
                    r.id = mock_finalised_request_requisition_line().id;
                }),
            ),
            Err(ServiceError::CannotEditRequisition)
        );

        // NotAResponseRequisition
        assert_eq!(
            service.update_response_requisition_line(
                &context,
                "store_a",
                "n/a",
                inline_init(|r: &mut UpdateResponseRequisitionLine| {
                    r.id = mock_sent_request_requisition_line().id;
                }),
            ),
            Err(ServiceError::NotAResponseRequisition)
        );
    }

    #[actix_rt::test]
    async fn update_response_requisition_line_success() {
        let (_, connection, connection_manager, _) = setup_all(
            "update_response_requisition_line_success",
            MockDataInserts::all(),
        )
        .await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.requisition_line_service;

        let test_line = mock_new_response_requisition_test().lines[0].clone();

        service
            .update_response_requisition_line(
                &context,
                "store_a",
                &mock_user_account_b().id,
                UpdateResponseRequisitionLine {
                    id: test_line.id.clone(),
                    supply_quantity: Some(99),
                    comment: Some("comment".to_string()),
                },
            )
            .unwrap();

        let line = RequisitionLineRowRepository::new(&connection)
            .find_one_by_id(&test_line.id)
            .unwrap()
            .unwrap();

        assert_eq!(
            line,
            inline_edit(&test_line, |mut u| {
                u.supply_quantity = 99;
                u.comment = Some("comment".to_string());
                u
            })
        );

        let requisition = RequisitionRowRepository::new(&connection)
            .find_one_by_id(&mock_new_response_requisition_test().requisition.id)
            .unwrap()
            .unwrap();

        assert_eq!(
            requisition,
            inline_edit(&requisition, |mut u| {
                u.user_id = Some(mock_user_account_b().id);
                u
            })
        );
    }
}
