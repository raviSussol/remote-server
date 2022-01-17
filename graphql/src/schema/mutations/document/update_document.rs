use async_graphql::*;
use chrono::{DateTime, Utc};
use service::{
    document::{
        document_service::{DocumentInsertError, DocumentService},
        raw_document::RawDocument,
    },
    permission_validation::{Resource, ResourceAccessRequest},
};

use crate::{
    schema::types::{DocumentNode, RawDocumentNode},
    standard_graphql_error::{validate_auth, StandardGraphqlError},
    ContextExt,
};

#[derive(InputObject)]
pub struct UpdateDocumentInput {
    pub name: String,
    pub parents: Vec<String>,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub type_: String,
    pub data: serde_json::Value,
    pub schema_id: Option<String>,
}

pub struct MergeRequiredError(Option<RawDocument>);
#[Object]
impl MergeRequiredError {
    pub async fn description(&self) -> &'static str {
        "Merge required"
    }

    pub async fn auto_merge(&self) -> Option<RawDocumentNode> {
        self.0.as_ref().map(|document| RawDocumentNode {
            document: document.clone(),
        })
    }
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "String"))]
pub enum UpdateDocumentErrorInterface {
    MergeRequired(MergeRequiredError),
}

#[derive(SimpleObject)]
pub struct UpdateDocumentError {
    pub error: UpdateDocumentErrorInterface,
}

#[derive(Union)]
pub enum UpdateDocumentResponse {
    Error(UpdateDocumentError),
    Response(DocumentNode),
}

pub fn update_document(
    ctx: &Context<'_>,
    store_id: &str,
    input: UpdateDocumentInput,
) -> Result<UpdateDocumentResponse> {
    validate_auth(
        ctx,
        &ResourceAccessRequest {
            resource: Resource::UpdateDocument,
            store_id: Some(store_id.to_string()),
        },
    )?;

    let connection_manager = ctx.get_connection_manager();
    let connection = connection_manager.connection()?;
    let service = DocumentService::new(&connection);

    let response = match service.insert_document(store_id, input_to_raw_document(input)) {
        Ok(document) => UpdateDocumentResponse::Response(DocumentNode { document }),
        Err(error) => UpdateDocumentResponse::Error(UpdateDocumentError {
            error: map_error(error)?,
        }),
    };
    Ok(response)
}

fn map_error(error: DocumentInsertError) -> Result<UpdateDocumentErrorInterface> {
    let formatted_error = format!("{:#?}", error);

    let graphql_error = match error {
        // Structured Errors
        DocumentInsertError::MergeRequired(auto_merge) => {
            return Ok(UpdateDocumentErrorInterface::MergeRequired(
                MergeRequiredError(auto_merge),
            ))
        }
        // Standard Graphql Errors
        DocumentInsertError::DatabaseError(_) => {
            StandardGraphqlError::InternalError(formatted_error)
        }
        DocumentInsertError::InvalidDataSchema(_) => {
            StandardGraphqlError::BadUserInput(formatted_error)
        }
        DocumentInsertError::InvalidDocumentHistory => {
            StandardGraphqlError::InternalError(formatted_error)
        }
        DocumentInsertError::FinalisationError(_) => {
            StandardGraphqlError::InternalError(formatted_error)
        }
        DocumentInsertError::InternalError(_) => {
            StandardGraphqlError::InternalError(formatted_error)
        }
    };

    Err(graphql_error.extend())
}

fn input_to_raw_document(
    UpdateDocumentInput {
        name,
        parents,
        author,
        timestamp,
        type_,
        data,
        schema_id,
    }: UpdateDocumentInput,
) -> RawDocument {
    RawDocument {
        name,
        parents,
        author,
        timestamp,
        type_,
        data,
        schema_id,
    }
}
