use crate::schema::types::DocumentNode;
use crate::standard_graphql_error::validate_auth;
use crate::ContextExt;
use async_graphql::*;
use service::document::document_service::{DocumentService, DocumentServiceTrait};
use service::permission_validation::{Resource, ResourceAccessRequest};

#[derive(Union)]
pub enum DocumentResponse {
    Response(DocumentNode),
}

pub fn document(ctx: &Context<'_>, store_id: String, name: String) -> Result<DocumentResponse> {
    validate_auth(
        ctx,
        &ResourceAccessRequest {
            resource: Resource::GetDocument,
            store_id: Some(store_id.to_string()),
        },
    )?;

    let service_provider = ctx.service_provider();
    let context = service_provider.context()?;
    let service = DocumentService {};

    let document = service.get_document(&context, &store_id, &name)?;
    Ok(DocumentResponse::Response(DocumentNode { document }))
}
