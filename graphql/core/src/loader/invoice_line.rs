use actix_web::web::Data;
use async_graphql::dataloader::*;
use async_graphql::*;
use repository::EqualFilter;
use repository::{InvoiceLine, InvoiceLineFilter};
use service::service_provider::ServiceProvider;
use std::collections::HashMap;

use super::{IdPairWithPayload, RequisitionAndItemId};

pub struct InvoiceLineByInvoiceIdLoader {
    pub service_provider: Data<ServiceProvider>,
}

#[async_trait::async_trait]
impl Loader<String> for InvoiceLineByInvoiceIdLoader {
    type Value = Vec<InvoiceLine>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        invoice_ids: &[String],
    ) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let service_context = self.service_provider.context()?;

        let filter = InvoiceLineFilter::new().invoice_id(EqualFilter::equal_any(
            invoice_ids.iter().map(String::clone).collect(),
        ));

        let invoice_lines = self
            .service_provider
            .invoice_line_service
            .get_invoice_lines(&service_context, Some(filter))?;

        let mut map: HashMap<String, Vec<InvoiceLine>> = HashMap::new();
        for line in invoice_lines {
            let list = map
                .entry(line.invoice_line_row.invoice_id.clone())
                .or_insert_with(|| Vec::<InvoiceLine>::new());
            list.push(line);
        }
        Ok(map)
    }
}

pub struct InvoiceLineForRequisitionLine {
    pub service_provider: Data<ServiceProvider>,
}

#[async_trait::async_trait]
impl Loader<RequisitionAndItemId> for InvoiceLineForRequisitionLine {
    type Value = Vec<InvoiceLine>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        requisition_and_item_id: &[RequisitionAndItemId],
    ) -> Result<HashMap<RequisitionAndItemId, Self::Value>, Self::Error> {
        let service_context = self.service_provider.context()?;

        let (requisition_ids, item_ids) =
            IdPairWithPayload::extract_unique_ids(requisition_and_item_id);

        let filter = InvoiceLineFilter::new()
            .requisition_id(EqualFilter::equal_any(requisition_ids))
            .item_id(EqualFilter::equal_any(item_ids));

        let invoice_lines = self
            .service_provider
            .invoice_line_service
            .get_invoice_lines(&service_context, Some(filter))?;

        let mut map = HashMap::new();
        for line in invoice_lines {
            if let Some(requisition_id) = &line.invoice_row.requisition_id {
                let list = map
                    .entry(RequisitionAndItemId::new(
                        &requisition_id,
                        &line.invoice_line_row.item_id,
                    ))
                    .or_insert_with(|| Vec::<InvoiceLine>::new());
                list.push(line);
            }
        }
        Ok(map)
    }
}
