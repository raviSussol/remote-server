mod generate;
mod validate;

use generate::generate;
use repository::{InvoiceLine, InvoiceLineRowRepository, RepositoryError};
use validate::validate;

use crate::{
    invoice_line::{query::get_invoice_line, ShipmentTaxUpdate},
    service_provider::ServiceContext,
    WithDBError,
};
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UpdateInboundShipmentServiceLine {
    pub id: String,
    pub invoice_id: String,
    pub item_id: Option<String>,
    pub name: Option<String>,
    pub total_before_tax: Option<f64>,
    pub total_after_tax: Option<f64>,
    pub tax: Option<ShipmentTaxUpdate>,
    pub note: Option<String>,
}

type OutError = UpdateInboundShipmentServiceLineError;

pub fn update_inbound_shipment_service_line(
    ctx: &ServiceContext,
    _store_id: &str,
    input: UpdateInboundShipmentServiceLine,
) -> Result<InvoiceLine, OutError> {
    let updated_line = ctx
        .connection
        .transaction_sync(|connection| {
            let (existing_line, _, item) = validate(&input, &connection)?;
            let updated_line = generate(input, existing_line, item)?;
            InvoiceLineRowRepository::new(&connection).upsert_one(&updated_line)?;

            get_invoice_line(ctx, &updated_line.id)
                .map_err(|error| OutError::DatabaseError(error))?
                .ok_or(OutError::UpdatedLineDoesNotExist)
        })
        .map_err(|error| error.to_inner_error())?;

    Ok(updated_line)
}

#[derive(Debug, PartialEq)]
pub enum UpdateInboundShipmentServiceLineError {
    LineDoesNotExist,
    InvoiceDoesNotExist,
    NotAnInboundShipment,
    // NotThisStoreInvoice,
    NotThisInvoiceLine(String),
    CannotEditInvoice,
    ItemNotFound,
    NotAServiceItem,
    // Internal
    UpdatedLineDoesNotExist,
    DatabaseError(RepositoryError),
}

impl From<RepositoryError> for UpdateInboundShipmentServiceLineError {
    fn from(error: RepositoryError) -> Self {
        UpdateInboundShipmentServiceLineError::DatabaseError(error)
    }
}

impl<ERR> From<WithDBError<ERR>> for UpdateInboundShipmentServiceLineError
where
    ERR: Into<UpdateInboundShipmentServiceLineError>,
{
    fn from(result: WithDBError<ERR>) -> Self {
        match result {
            WithDBError::DatabaseError(error) => error.into(),
            WithDBError::Error(error) => error.into(),
        }
    }
}

#[cfg(test)]
mod test {

    use repository::{
        mock::{
            mock_default_service_item, mock_draft_inbound_service_line,
            mock_draft_inbound_shipment_no_lines, mock_draft_inbound_shipment_with_service_lines,
            mock_draft_inbound_verified_service_line,
            mock_draft_inbound_verified_with_service_lines, mock_draft_outbound_service_line,
            mock_draft_outbound_with_service_lines, mock_item_a, mock_item_service_item,
            MockDataInserts,
        },
        test_db::setup_all,
        InvoiceLineRowRepository,
    };
    use util::{inline_edit, inline_init};

    use crate::{
        invoice_line::{
            inbound_shipment_service_line::UpdateInboundShipmentServiceLine, ShipmentTaxUpdate,
        },
        service_provider::ServiceProvider,
    };

    use super::UpdateInboundShipmentServiceLineError;

    type ServiceError = UpdateInboundShipmentServiceLineError;

    #[actix_rt::test]
    async fn update_inbound_shipment_service_line_errors() {
        let (_, _, connection_manager, _) = setup_all(
            "update_inbound_shipment_service_line_errors",
            MockDataInserts::all(),
        )
        .await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.invoice_line_service;

        // LineDoesNotExist
        assert_eq!(
            service.update_inbound_shipment_service_line(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateInboundShipmentServiceLine| {
                    r.id = "invalid".to_string();
                }),
            ),
            Err(ServiceError::LineDoesNotExist)
        );

        // InvoiceDoesNotExist
        assert_eq!(
            service.update_inbound_shipment_service_line(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateInboundShipmentServiceLine| {
                    r.id = mock_draft_inbound_service_line().id;
                    r.invoice_id = "invalid".to_string();
                }),
            ),
            Err(ServiceError::InvoiceDoesNotExist)
        );

        // NotAnInboundShipment
        assert_eq!(
            service.update_inbound_shipment_service_line(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateInboundShipmentServiceLine| {
                    r.invoice_id = mock_draft_outbound_with_service_lines().id;
                    r.id = mock_draft_outbound_service_line().id;
                }),
            ),
            Err(ServiceError::NotAnInboundShipment)
        );

        // NotThisInvoiceLine
        assert_eq!(
            service.update_inbound_shipment_service_line(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateInboundShipmentServiceLine| {
                    r.id = mock_draft_inbound_service_line().id;
                    r.invoice_id = mock_draft_inbound_shipment_no_lines().id;
                }),
            ),
            Err(ServiceError::NotThisInvoiceLine(
                mock_draft_inbound_shipment_with_service_lines().id
            ))
        );

        // CannotEditInvoice
        assert_eq!(
            service.update_inbound_shipment_service_line(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateInboundShipmentServiceLine| {
                    r.id = mock_draft_inbound_verified_service_line().id;
                    r.invoice_id = mock_draft_inbound_verified_with_service_lines().id;
                }),
            ),
            Err(ServiceError::CannotEditInvoice)
        );

        // ItemNotFound
        assert_eq!(
            service.update_inbound_shipment_service_line(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateInboundShipmentServiceLine| {
                    r.id = mock_draft_inbound_service_line().id;
                    r.invoice_id = mock_draft_inbound_shipment_with_service_lines().id;
                    r.item_id = Some("invalid".to_string())
                }),
            ),
            Err(ServiceError::ItemNotFound)
        );

        // NotAServiceItem
        assert_eq!(
            service.update_inbound_shipment_service_line(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateInboundShipmentServiceLine| {
                    r.id = mock_draft_inbound_service_line().id;
                    r.invoice_id = mock_draft_inbound_shipment_with_service_lines().id;
                    r.item_id = Some(mock_item_a().id)
                }),
            ),
            Err(ServiceError::NotAServiceItem)
        );
    }

    #[actix_rt::test]
    async fn update_inbound_shipment_service_line_success() {
        let (_, connection, connection_manager, _) = setup_all(
            "update_inbound_shipment_service_line_service",
            MockDataInserts::all(),
        )
        .await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.invoice_line_service;

        // Service Item Changed
        service
            .update_inbound_shipment_service_line(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateInboundShipmentServiceLine| {
                    r.id = mock_draft_inbound_service_line().id;
                    r.invoice_id = mock_draft_inbound_shipment_with_service_lines().id;
                    r.item_id = Some(mock_item_service_item().id);
                }),
            )
            .unwrap();

        let line = InvoiceLineRowRepository::new(&connection)
            .find_one_by_id_option(&mock_draft_inbound_service_line().id)
            .unwrap()
            .unwrap();

        assert_eq!(line.item_name, mock_item_service_item().name);

        // Service Item Changed And Name updated
        service
            .update_inbound_shipment_service_line(
                &context,
                "store_a",
                inline_init(|r: &mut UpdateInboundShipmentServiceLine| {
                    r.id = mock_draft_inbound_service_line().id;
                    r.invoice_id = mock_draft_inbound_shipment_with_service_lines().id;
                    r.item_id = Some(mock_default_service_item().id);
                    r.name = Some("name".to_string());
                }),
            )
            .unwrap();

        let line = InvoiceLineRowRepository::new(&connection)
            .find_one_by_id_option(&mock_draft_inbound_service_line().id)
            .unwrap()
            .unwrap();

        assert_eq!(line.item_name, "name");

        // All fields

        service
            .update_inbound_shipment_service_line(
                &context,
                "store_a",
                UpdateInboundShipmentServiceLine {
                    id: mock_draft_inbound_service_line().id,
                    invoice_id: mock_draft_inbound_shipment_with_service_lines().id,
                    item_id: Some(mock_item_service_item().id),
                    name: Some("modified name".to_string()),
                    total_before_tax: Some(1.0),
                    total_after_tax: Some(1.1),
                    tax: Some(ShipmentTaxUpdate {
                        percentage: Some(10.0),
                    }),
                    note: Some("note".to_string()),
                },
            )
            .unwrap();

        let line = InvoiceLineRowRepository::new(&connection)
            .find_one_by_id_option(&mock_draft_inbound_service_line().id)
            .unwrap()
            .unwrap();

        assert_eq!(
            line,
            inline_edit(&line, |mut u| {
                u.invoice_id = mock_draft_inbound_shipment_with_service_lines().id;
                u.item_id = mock_item_service_item().id;
                u.item_name = "modified name".to_string();
                u.total_before_tax = 1.0;
                u.total_after_tax = 1.1;
                u.tax = Some(10.0);
                u.note = Some("note".to_string());
                u
            })
        );
    }
}
