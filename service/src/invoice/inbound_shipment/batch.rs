use repository::{Invoice, InvoiceLine, RepositoryError};

use crate::{
    invoice_line::{
        inbound_shipment_line::{
            delete_inbound_shipment_line, insert_inbound_shipment_line,
            update_inbound_shipment_line, DeleteInboundShipmentLine,
            DeleteInboundShipmentLineError, InsertInboundShipmentLine,
            InsertInboundShipmentLineError, UpdateInboundShipmentLine,
            UpdateInboundShipmentLineError,
        },
        inbound_shipment_service_line::{
            delete_inbound_shipment_service_line, insert_inbound_shipment_service_line,
            update_inbound_shipment_service_line, DeleteInboundShipmentServiceLineError,
            InsertInboundShipmentServiceLine, InsertInboundShipmentServiceLineError,
            UpdateInboundShipmentServiceLine, UpdateInboundShipmentServiceLineError,
        },
    },
    service_provider::ServiceContext,
    BatchMutationsProcessor, InputWithResult, WithDBError,
};

use super::{
    delete_inbound_shipment, insert_inbound_shipment, update_inbound_shipment,
    DeleteInboundShipment, DeleteInboundShipmentError, InsertInboundShipment,
    InsertInboundShipmentError, UpdateInboundShipment, UpdateInboundShipmentError,
};

#[derive(Clone)]
pub struct BatchInboundShipment {
    pub insert_shipment: Option<Vec<InsertInboundShipment>>,
    pub insert_line: Option<Vec<InsertInboundShipmentLine>>,
    pub update_line: Option<Vec<UpdateInboundShipmentLine>>,
    pub delete_line: Option<Vec<DeleteInboundShipmentLine>>,
    pub insert_service_line: Option<Vec<InsertInboundShipmentServiceLine>>,
    pub update_service_line: Option<Vec<UpdateInboundShipmentServiceLine>>,
    pub delete_service_line: Option<Vec<DeleteInboundShipmentLine>>,
    pub update_shipment: Option<Vec<UpdateInboundShipment>>,
    pub delete_shipment: Option<Vec<DeleteInboundShipment>>,
    pub continue_on_error: Option<bool>,
}

pub type InsertShipmentsResult =
    Vec<InputWithResult<InsertInboundShipment, Result<Invoice, InsertInboundShipmentError>>>;
pub type InsertLinesResult = Vec<
    InputWithResult<InsertInboundShipmentLine, Result<InvoiceLine, InsertInboundShipmentLineError>>,
>;
pub type UpdateLinesResult = Vec<
    InputWithResult<UpdateInboundShipmentLine, Result<InvoiceLine, UpdateInboundShipmentLineError>>,
>;
pub type DeleteLinesResult =
    Vec<InputWithResult<DeleteInboundShipmentLine, Result<String, DeleteInboundShipmentLineError>>>;
pub type InsertServiceLinesResult = Vec<
    InputWithResult<
        InsertInboundShipmentServiceLine,
        Result<InvoiceLine, InsertInboundShipmentServiceLineError>,
    >,
>;
pub type UpdateServiceLinesResult = Vec<
    InputWithResult<
        UpdateInboundShipmentServiceLine,
        Result<InvoiceLine, UpdateInboundShipmentServiceLineError>,
    >,
>;
pub type DeleteServiceLinesResult = Vec<
    InputWithResult<
        DeleteInboundShipmentLine,
        Result<String, DeleteInboundShipmentServiceLineError>,
    >,
>;
pub type UpdateShipmentsResult =
    Vec<InputWithResult<UpdateInboundShipment, Result<Invoice, UpdateInboundShipmentError>>>;
pub type DeleteShipmentsResult =
    Vec<InputWithResult<DeleteInboundShipment, Result<String, DeleteInboundShipmentError>>>;

#[derive(Debug, Default)]
pub struct BatchInboundShipmentResult {
    pub insert_shipment: InsertShipmentsResult,
    pub insert_line: InsertLinesResult,
    pub update_line: UpdateLinesResult,
    pub delete_line: DeleteLinesResult,
    pub insert_service_line: InsertServiceLinesResult,
    pub update_service_line: UpdateServiceLinesResult,
    pub delete_service_line: DeleteServiceLinesResult,
    pub update_shipment: UpdateShipmentsResult,
    pub delete_shipment: DeleteShipmentsResult,
}

pub fn batch_inbound_shipment(
    ctx: &ServiceContext,
    store_id: &str,
    user_id: &str,
    input: BatchInboundShipment,
) -> Result<BatchInboundShipmentResult, RepositoryError> {
    let result = ctx
        .connection
        .transaction_sync(|_| {
            let continue_on_error = input.continue_on_error.unwrap_or(false);
            let mut results = BatchInboundShipmentResult::default();

            let mutations_processor = BatchMutationsProcessor::new(ctx, store_id, user_id);

            // Insert Shipment

            let (has_errors, result) = mutations_processor
                .do_mutations_with_user_id(input.insert_shipment, insert_inbound_shipment);
            results.insert_shipment = result;
            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            // Normal Line

            let (has_errors, result) = mutations_processor
                .do_mutations_with_user_id(input.insert_line, insert_inbound_shipment_line);
            results.insert_line = result;
            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            let (has_errors, result) = mutations_processor
                .do_mutations_with_user_id(input.update_line, update_inbound_shipment_line);
            results.update_line = result;
            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            let (has_errors, result) = mutations_processor
                .do_mutations_with_user_id(input.delete_line, delete_inbound_shipment_line);
            results.delete_line = result;
            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            // Service Line

            let (has_errors, result) = mutations_processor.do_mutations(
                input.insert_service_line,
                insert_inbound_shipment_service_line,
            );
            results.insert_service_line = result;
            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            let (has_errors, result) = mutations_processor.do_mutations(
                input.update_service_line,
                update_inbound_shipment_service_line,
            );
            results.update_service_line = result;
            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            let (has_errors, result) = mutations_processor.do_mutations(
                input.delete_service_line,
                delete_inbound_shipment_service_line,
            );
            results.delete_service_line = result;
            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            // Update and delete shipment

            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            let (has_errors, result) = mutations_processor
                .do_mutations_with_user_id(input.update_shipment, update_inbound_shipment);
            results.update_shipment = result;
            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            let (has_errors, result) = mutations_processor
                .do_mutations_with_user_id(input.delete_shipment, delete_inbound_shipment);
            results.delete_shipment = result;
            if has_errors && !continue_on_error {
                return Err(WithDBError::err(results));
            }

            Ok(results)
                as Result<BatchInboundShipmentResult, WithDBError<BatchInboundShipmentResult>>
        })
        .map_err(|error| error.to_inner_error())
        .or_else(|error| match error {
            WithDBError::DatabaseError(repository_error) => Err(repository_error),
            WithDBError::Error(batch_response) => Ok(batch_response),
        })?;

    Ok(result)
}

#[cfg(test)]
mod test {
    use repository::{
        mock::{mock_item_a, mock_name_a, mock_outbound_shipment_b, MockDataInserts},
        test_db::setup_all,
        InvoiceLineRowRepository, InvoiceRowRepository,
    };
    use util::inline_init;

    use crate::{
        invoice::inbound_shipment::{
            BatchInboundShipment, DeleteInboundShipment, DeleteInboundShipmentError,
            InsertInboundShipment,
        },
        invoice_line::inbound_shipment_line::InsertInboundShipmentLine,
        service_provider::ServiceProvider,
        InputWithResult,
    };

    #[actix_rt::test]
    async fn batch_inbound_shipment_service() {
        let (_, connection, connection_manager, _) =
            setup_all("batch_inbound_shipment_service", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.invoice_service;

        let delete_shipment_input = inline_init(|input: &mut DeleteInboundShipment| {
            input.id = mock_outbound_shipment_b().id;
        });

        let mut input = BatchInboundShipment {
            insert_shipment: Some(vec![inline_init(|input: &mut InsertInboundShipment| {
                input.id = "new_id".to_string();
                input.other_party_id = mock_name_a().id;
            })]),
            insert_line: Some(vec![inline_init(
                |input: &mut InsertInboundShipmentLine| {
                    input.invoice_id = "new_id".to_string();
                    input.id = "new_line_id".to_string();
                    input.item_id = mock_item_a().id;
                    input.pack_size = 1;
                    input.number_of_packs = 1;
                },
            )]),
            update_line: None,
            delete_line: None,
            update_shipment: None,
            delete_shipment: Some(vec![delete_shipment_input.clone()]),
            continue_on_error: None,
            insert_service_line: None,
            update_service_line: None,
            delete_service_line: None,
        };

        // Test rollback
        let result = service
            .batch_inbound_shipment(&context, "store_a", "n/a", input.clone())
            .unwrap();

        assert_eq!(
            result.delete_shipment,
            vec![InputWithResult {
                input: delete_shipment_input,
                result: Err(DeleteInboundShipmentError::NotAnInboundShipment {})
            }]
        );

        assert_eq!(
            InvoiceRowRepository::new(&connection)
                .find_one_by_id_option("new_id")
                .unwrap(),
            None
        );

        assert_eq!(
            InvoiceLineRowRepository::new(&connection)
                .find_one_by_id_option("new_line_id")
                .unwrap(),
            None
        );

        // Test no rollback
        input.continue_on_error = Some(true);

        service
            .batch_inbound_shipment(&context, "store_a", "n/a", input)
            .unwrap();

        assert_ne!(
            InvoiceRowRepository::new(&connection)
                .find_one_by_id_option("new_id")
                .unwrap(),
            None
        );

        assert_ne!(
            InvoiceLineRowRepository::new(&connection)
                .find_one_by_id_option("new_line_id")
                .unwrap(),
            None
        );
    }
}
