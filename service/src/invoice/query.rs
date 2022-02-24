use crate::{
    get_default_pagination, i64_to_u32, service_provider::ServiceContext, ListError, ListResult,
    SingleRecordError,
};
use domain::{EqualFilter, PaginationOption};
use repository::{
    schema::InvoiceRowType, Invoice, InvoiceFilter, InvoiceQueryRepository, InvoiceSort,
    RepositoryError, StorageConnectionManager,
};

pub const MAX_LIMIT: u32 = 1000;
pub const MIN_LIMIT: u32 = 1;

pub fn get_invoices(
    connection_manager: &StorageConnectionManager,
    store_id_option: Option<&str>,
    pagination: Option<PaginationOption>,
    filter: Option<InvoiceFilter>,
    sort: Option<InvoiceSort>,
) -> Result<ListResult<Invoice>, ListError> {
    let pagination = get_default_pagination(pagination, MAX_LIMIT, MIN_LIMIT)?;
    let connection = connection_manager.connection()?;
    let repository = InvoiceQueryRepository::new(&connection);

    let mut filter = filter.unwrap_or(InvoiceFilter::new());
    filter.store_id = store_id_option.map(EqualFilter::equal_to);

    Ok(ListResult {
        rows: repository.query(pagination, Some(filter.clone()), sort)?,
        count: i64_to_u32(repository.count(Some(filter))?),
    })
}

pub fn get_invoice(
    connection_manager: &StorageConnectionManager,
    store_id_option: Option<&str>,
    id: String,
) -> Result<Invoice, SingleRecordError> {
    let connection = connection_manager.connection()?;

    let mut filter = InvoiceFilter::new().id(EqualFilter::equal_to(&id));
    filter.store_id = store_id_option.map(EqualFilter::equal_to);

    let mut result = InvoiceQueryRepository::new(&connection).query_by_filter(filter)?;

    if let Some(record) = result.pop() {
        Ok(record)
    } else {
        Err(SingleRecordError::NotFound(id))
    }
}

pub fn get_invoice_by_number(
    ctx: &ServiceContext,
    store_id: &str,
    invoice_number: u32,
    r#type: InvoiceRowType,
) -> Result<Option<Invoice>, RepositoryError> {
    let mut result = InvoiceQueryRepository::new(&ctx.connection).query_by_filter(
        InvoiceFilter::new()
            .invoice_number(EqualFilter::equal_to_i64(invoice_number as i64))
            .store_id(EqualFilter::equal_to(store_id))
            .r#type(r#type.equal_to()),
    )?;

    Ok(result.pop())
}

#[cfg(test)]
mod test_query {
    use repository::{
        mock::{mock_unique_number_inbound_shipment, MockDataInserts},
        schema::InvoiceRowType,
        test_db::setup_all,
    };

    use crate::service_provider::ServiceProvider;

    #[actix_rt::test]
    async fn get_invoice_by_number() {
        let (_, _, connection_manager, _) =
            setup_all("get_invoice_by_number", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.invoice_service;

        // Not found
        assert_eq!(
            service.get_invoice_by_number(
                &context,
                "store_a",
                200,
                InvoiceRowType::OutboundShipment
            ),
            Ok(None)
        );

        let invoice_to_find = mock_unique_number_inbound_shipment();

        // Not found - wrong type
        assert_eq!(
            service.get_invoice_by_number(
                &context,
                "store_a",
                invoice_to_find.invoice_number as u32,
                InvoiceRowType::OutboundShipment,
            ),
            Ok(None)
        );

        // Found
        let found_invoice = service
            .get_invoice_by_number(
                &context,
                "store_a",
                invoice_to_find.invoice_number as u32,
                InvoiceRowType::InboundShipment,
            )
            .unwrap()
            .unwrap();

        assert_eq!(found_invoice.invoice_row.id, invoice_to_find.id);
    }
}
