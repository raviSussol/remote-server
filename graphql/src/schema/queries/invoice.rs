use async_graphql::*;
use repository::StorageConnectionManager;
use service::invoice::get_invoice as get_invoice_service;

use crate::{
    schema::types::{
        invoice_query::{InvoiceNode, InvoiceNodeType},
        NodeError, NodeErrorInterface,
    },
    ContextExt,
};

#[derive(Union)]
pub enum InvoiceResponse {
    Error(NodeError),
    Response(InvoiceNode),
}

pub fn get_invoice(
    connection_manager: &StorageConnectionManager,
    store_id: Option<&str>,
    id: String,
) -> InvoiceResponse {
    match get_invoice_service(connection_manager, store_id, id) {
        Ok(invoice) => InvoiceResponse::Response(InvoiceNode::from_domain(invoice)),
        Err(error) => InvoiceResponse::Error(error.into()),
    }
}

pub fn get_invoice_by_number(
    ctx: &Context<'_>,
    store_id: &str,
    invoice_number: u32,
    r#type: InvoiceNodeType,
) -> Result<InvoiceResponse> {
    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;
    let invoice_service = &service_provider.invoice_service;

    let invoice_option = invoice_service.get_invoice_by_number(
        &service_context,
        store_id,
        invoice_number,
        r#type.into(),
    )?;

    let response = match invoice_option {
        Some(invoice) => InvoiceResponse::Response(InvoiceNode::from_domain(invoice)),
        None => InvoiceResponse::Error(NodeError {
            error: NodeErrorInterface::record_not_found(),
        }),
    };

    Ok(response)
}
