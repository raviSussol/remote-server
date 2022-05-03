#[cfg(test)]
mod test_update {
    use chrono::{NaiveDate, Utc};
    use repository::{
        mock::{
            mock_draft_request_requisition_for_update_test,
            mock_draft_response_requisition_for_update_test, mock_name_store_c,
            mock_request_draft_requisition_calculation_test, mock_sent_request_requisition,
            mock_store_a, MockData, MockDataInserts,
        },
        requisition_row::RequisitionRowStatus,
        schema::{NameRow, NameStoreJoinRow},
        test_db::{setup_all, setup_all_with_data},
        RequisitionLineRowRepository, RequisitionRowRepository,
    };
    use util::{inline_edit, inline_init};

    use crate::{
        requisition::request_requisition::{
            UpdateRequestRequisition, UpdateRequestRequisitionError as ServiceError,
            UpdateRequestRequstionStatus,
        },
        service_provider::ServiceProvider,
    };

    #[actix_rt::test]
    async fn update_request_requisition_errors() {
        fn not_visible() -> NameRow {
            inline_init(|r: &mut NameRow| {
                r.id = "not_visible".to_string();
            })
        }

        fn not_a_supplier() -> NameRow {
            inline_init(|r: &mut NameRow| {
                r.id = "not_a_supplier".to_string();
            })
        }

        fn not_a_supplier_join() -> NameStoreJoinRow {
            inline_init(|r: &mut NameStoreJoinRow| {
                r.id = "not_a_supplier_join".to_string();
                r.name_id = not_a_supplier().id;
                r.store_id = mock_store_a().id;
                r.name_is_supplier = false;
            })
        }

        let (_, _, connection_manager, _) = setup_all_with_data(
            "update_request_requisition_errors",
            MockDataInserts::all(),
            inline_init(|r: &mut MockData| {
                r.names = vec![not_visible(), not_a_supplier()];
                r.name_store_joins = vec![not_a_supplier_join()];
            }),
        )
        .await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.requisition_service;

        // RequisitionDoesNotExist
        assert_eq!(
            service.update_request_requisition(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateRequestRequisition| {
                    r.id = "invalid".to_owned();
                }),
            ),
            Err(ServiceError::RequisitionDoesNotExist)
        );

        // NotThisStoreRequisition
        assert_eq!(
            service.update_request_requisition(
                &context,
                "store_b",
                inline_init(|r: &mut UpdateRequestRequisition| {
                    r.id = mock_draft_request_requisition_for_update_test().id;
                }),
            ),
            Err(ServiceError::NotThisStoreRequisition)
        );

        // CannotEditRequisition
        assert_eq!(
            service.update_request_requisition(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateRequestRequisition| {
                    r.id = mock_sent_request_requisition().id;
                }),
            ),
            Err(ServiceError::CannotEditRequisition)
        );

        // NotARequestRequisition
        assert_eq!(
            service.update_request_requisition(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateRequestRequisition| {
                    r.id = mock_draft_response_requisition_for_update_test().id;
                }),
            ),
            Err(ServiceError::NotARequestRequisition)
        );

        // OtherPartyDoesNotExist
        assert_eq!(
            service.update_request_requisition(
                &context,
                &mock_store_a().id,
                inline_init(|r: &mut UpdateRequestRequisition| {
                    r.id = mock_draft_request_requisition_for_update_test().id;
                    r.other_party_id = Some("invalid".to_string());
                })
            ),
            Err(ServiceError::OtherPartyDoesNotExist)
        );
        // OtherPartyNotVisible
        assert_eq!(
            service.update_request_requisition(
                &context,
                &mock_store_a().id,
                inline_init(|r: &mut UpdateRequestRequisition| {
                    r.id = mock_draft_request_requisition_for_update_test().id;
                    r.other_party_id = Some(not_visible().id);
                })
            ),
            Err(ServiceError::OtherPartyNotVisible)
        );
        // OtherPartyNotASupplier
        assert_eq!(
            service.update_request_requisition(
                &context,
                &mock_store_a().id,
                inline_init(|r: &mut UpdateRequestRequisition| {
                    r.id = mock_draft_request_requisition_for_update_test().id;
                    r.other_party_id = Some(not_a_supplier().id);
                })
            ),
            Err(ServiceError::OtherPartyNotASupplier)
        );
    }

    #[actix_rt::test]
    async fn update_request_requisition_success() {
        let (_, connection, connection_manager, _) =
            setup_all("update_request_requisition_success", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.requisition_service;

        let before_update = Utc::now().naive_utc();

        // Simple update (without recalculation)
        let result = service
            .update_request_requisition(
                &context,
                "store_a",
                UpdateRequestRequisition {
                    id: mock_draft_request_requisition_for_update_test().id,
                    colour: Some("new colour".to_owned()),
                    status: Some(UpdateRequestRequstionStatus::Sent),
                    their_reference: Some("new their_reference".to_owned()),
                    comment: Some("new comment".to_owned()),
                    max_months_of_stock: None,
                    min_months_of_stock: None,
                    other_party_id: Some(mock_name_store_c().id),
                    expected_delivery_date: Some(NaiveDate::from_ymd(2022, 01, 03)),
                },
            )
            .unwrap();

        let after_update = Utc::now().naive_utc();

        let updated_row = RequisitionRowRepository::new(&connection)
            .find_one_by_id(&result.requisition_row.id)
            .unwrap()
            .unwrap();

        assert_eq!(
            updated_row,
            inline_edit(&updated_row, |mut u| {
                u.colour = Some("new colour".to_owned());
                u.status = RequisitionRowStatus::Sent;
                u.their_reference = Some("new their_reference".to_owned());
                u.comment = Some("new comment".to_owned());
                u.name_id = mock_name_store_c().id;
                u.expected_delivery_date = Some(NaiveDate::from_ymd(2022, 01, 03));
                u
            })
        );

        let sent_datetime = updated_row.sent_datetime.unwrap();
        assert!(sent_datetime > before_update && sent_datetime < after_update);

        // Recalculate stock

        let calculation_requisition = mock_request_draft_requisition_calculation_test();

        service
            .update_request_requisition(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateRequestRequisition| {
                    r.id = calculation_requisition.requisition.id.clone();
                    r.max_months_of_stock = Some(20.0);
                }),
            )
            .unwrap();

        let requisition_line_row_repo = RequisitionLineRowRepository::new(&connection);

        // Calculated
        let line = requisition_line_row_repo
            .find_one_by_id(&calculation_requisition.lines[0].id)
            .unwrap()
            .unwrap();
        assert_eq!(line.suggested_quantity, 19);

        // Average monthly consumption = 0
        let line = requisition_line_row_repo
            .find_one_by_id(&calculation_requisition.lines[1].id)
            .unwrap()
            .unwrap();
        assert_eq!(line.suggested_quantity, 0);

        // Above threshold MOS
        let line = requisition_line_row_repo
            .find_one_by_id(&calculation_requisition.lines[2].id)
            .unwrap()
            .unwrap();
        assert_eq!(line.suggested_quantity, 0);

        // Above max MOS
        let line = requisition_line_row_repo
            .find_one_by_id(&calculation_requisition.lines[3].id)
            .unwrap()
            .unwrap();
        assert_eq!(line.suggested_quantity, 0);
    }
}
