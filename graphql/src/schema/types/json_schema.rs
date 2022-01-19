use async_graphql::*;
use domain::json_schema::JSONSchema;

pub struct JSONSchemaNode {
    pub schema: JSONSchema,
}

#[Object]
impl JSONSchemaNode {
    pub async fn id(&self) -> &str {
        &self.schema.id
    }

    pub async fn schema(&self) -> &serde_json::Value {
        &self.schema.schema
    }
}
