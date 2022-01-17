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
    schema::types::{DatabaseError, DocumentNode, JSONSchemaNode},
    standard_graphql_error::{validate_auth, StandardGraphqlError},
    ContextExt,
};

#[derive(InputObject)]
pub struct UpdateJsonSchemaInput {
    pub id: String,
    pub schema: String,
}

#[derive(Union)]
pub enum UpdateDocumentResponse {
    Response(JSONSchemaNode),
}

pub fn update_json_schema(
    ctx: &Context<'_>,
    store_id: &str,
    input: UpdateJsonSchemaInput,
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
                MergeRequiredError(auto_merge.map(|document| RawDocumentNode { document })),
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
