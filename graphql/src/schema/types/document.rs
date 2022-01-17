use async_graphql::*;
use async_graphql::{dataloader::DataLoader, Context};
use chrono::{DateTime, Utc};
use domain::document::Document;

use crate::{loader::JsonSchemaLoader, standard_graphql_error::StandardGraphqlError, ContextExt};

use super::JSONSchemaNode;

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

    pub async fn schema(&self, ctx: &Context<'_>) -> Result<Option<JSONSchemaNode>> {
        Ok(match &self.document.schema_id {
            Some(schema_id) => {
                let loader = ctx.get_loader::<DataLoader<JsonSchemaLoader>>();
                let schema = loader.load_one(schema_id.clone()).await?.ok_or(
                    StandardGraphqlError::InternalError(format!(
                        "Cannot find schema {}",
                        schema_id
                    ))
                    .extend(),
                )?;
                Some(JSONSchemaNode { schema })
            }
            None => None,
        })
    }
}
