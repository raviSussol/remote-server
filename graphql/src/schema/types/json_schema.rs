use async_graphql::*;
use domain::json_schema::JSONSchema;

use crate::standard_graphql_error::StandardGraphqlError;

pub struct JSONSchemaNode {
    pub schema: JSONSchema,
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
