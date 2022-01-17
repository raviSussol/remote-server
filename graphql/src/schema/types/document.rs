use async_graphql::*;
use chrono::{DateTime, Utc};
use domain::{document::Document, json_schema::JSONSchema};

use crate::standard_graphql_error::StandardGraphqlError;

pub struct JSONSchemaNode {
    schema: JSONSchema,
}

#[Object]
impl JSONSchemaNode {
    pub async fn id(&self) -> &str {
        &self.schema.id
    }

    pub async fn schema(&self) -> Result<String> {
        Ok(serde_json::to_string(&self.schema.schema).map_err(|e| {
            StandardGraphqlError::InternalError(format!("Failed to stringify json value: {}", e))
                .extend()
        })?)
    }
}

pub struct DocumentNode {
    pub document: Document,
}

#[Object]
impl DocumentNode {
    pub async fn id(&self) -> &str {
        &self.document.id
    }

    pub async fn name(&self) -> &str {
        &self.document.name
    }

    pub async fn parents(&self) -> &[String] {
        &self.document.parents
    }

    pub async fn author(&self) -> &str {
        &self.document.author
    }

    pub async fn timestamp(&self) -> &DateTime<Utc> {
        &self.document.timestamp
    }

    #[graphql(name = "type")]
    pub async fn type_(&self) -> &str {
        &self.document.type_
    }

    pub async fn data(&self) -> Result<String> {
        Ok(serde_json::to_string(&self.document.data).map_err(|e| {
            StandardGraphqlError::InternalError(format!("Failed to stringify json value: {}", e))
                .extend()
        })?)
    }

    pub async fn schema(&self) -> Result<Option<String>> {
        Ok(match &self.document.schema {
            Some(schema) => Some(serde_json::to_string(schema).map_err(|e| {
                StandardGraphqlError::InternalError(format!(
                    "Failed to stringify json value: {}",
                    e
                ))
                .extend()
            })?),
            None => None,
        })
    }
}
