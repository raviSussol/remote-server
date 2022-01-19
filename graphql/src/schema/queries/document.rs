use crate::schema::types::sort_filter_types::EqualFilterStringInput;
use crate::schema::types::{DocumentConnector, DocumentNode};
use crate::standard_graphql_error::validate_auth;
use crate::ContextExt;
use async_graphql::*;
use domain::EqualFilter;
use repository::DocumentFilter;
use service::document::document_service::{DocumentService, DocumentServiceTrait};
use service::permission_validation::{Resource, ResourceAccessRequest};
use service::usize_to_u32;

#[derive(Union)]
pub enum DocumentResponse {
    Response(DocumentConnector),
}

#[derive(InputObject, Clone)]
pub struct DocumentFilterInput {
    pub name: Option<EqualFilterStringInput>,
}
fn to_domain_filter(f: DocumentFilterInput) -> DocumentFilter {
    DocumentFilter {
        name: f.name.map(EqualFilter::from),
    }
}

pub fn documents(
    ctx: &Context<'_>,
    store_id: String,
    filter: Option<DocumentFilterInput>,
) -> Result<DocumentResponse> {
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

    let nodes: Vec<DocumentNode> = service
        .get_documents(&context, &store_id, filter.map(to_domain_filter))?
        .into_iter()
        .map(|document| DocumentNode { document })
        .collect();

    Ok(DocumentResponse::Response(DocumentConnector {
        total_count: usize_to_u32(nodes.len()),
        nodes,
    }))
}
