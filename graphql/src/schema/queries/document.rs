use crate::schema::types::DocumentNode;
use crate::standard_graphql_error::validate_auth;
use crate::ContextExt;
use async_graphql::*;
use service::document::document_service::DocumentService;
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

    let connection_manager = ctx.get_connection_manager();
    let connection = connection_manager.connection()?;
    let service = DocumentService::new(&connection);
    let document = service.get_document(&store_id, &name)?;
    Ok(DocumentResponse::Response(DocumentNode { document }))
}
